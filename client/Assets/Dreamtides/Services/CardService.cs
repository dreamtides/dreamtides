#nullable enable

using System.Collections;
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Masonry;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Services
{
  public class CardService : Service
  {
    [SerializeField] AudioClip? _shuffleVoidIntoDeckSound;
    Card? _currentInfoZoom;
    bool _hidCloseButton;
    bool _infoZoomDisabled;
    List<Card> _cardsWithInfoZoomIcons = new();
    Coroutine? _delayedIconClearCoroutine;

    public bool IsPointerDownOnCard { get; set; } = false;

    protected override void OnInitialize(TestConfiguration? testConfiguration)
    {
      _infoZoomDisabled = testConfiguration != null;
    }

    public IEnumerator HandleDrawUserCards(DrawUserCardsCommand command)
    {
      for (var i = 0; i < command.Cards.Count; ++i)
      {
        if (i < command.Cards.Count - 1)
        {
          StartCoroutine(DrawUserCard(command, i, isLastCard: false));
          yield return new WaitForSeconds(command.StaggerInterval.ToSeconds());
        }
        else
        {
          yield return DrawUserCard(command, i, isLastCard: true);
        }
      }
    }

    public IEnumerator HandleShuffleVoidIntoDeck(ShuffleVoidIntoDeckCommand command)
    {
      var (source, destination) = command.Player switch
      {
        DisplayPlayer.User => (Registry.Layout.UserVoid, Registry.Layout.UserDeck),
        DisplayPlayer.Enemy => (Registry.Layout.EnemyVoid, Registry.Layout.EnemyDeck),
        _ => (null, null)
      };

      if (source == null || destination == null)
      {
        yield break;
      }

      var sourceCards = source.Objects.OfType<Card>().ToList();
      if (sourceCards.Count == 0)
      {
        yield break;
      }

      const float totalDuration = 3f;
      const float earlyPhaseDuration = 1f; // First second: move only up to 10 cards
      float shuffleRotationDuration = 0.4f; // Wiggle at end (can be trimmed if overcrowded)
      float movePhaseDuration = Mathf.Max(0f, totalDuration - shuffleRotationDuration);

      int earlyBatchCount = Mathf.Min(10, sourceCards.Count);
      int laterBatchCount = sourceCards.Count - earlyBatchCount;

      // If there are too many cards, ensure later cards still have some minimum time slice.
      const float minLegDuration = 0.03f; // per leg minimal duration target
      const float earlyPauseFraction = 0.35f; // portion of early slice used for pause (tapered)

      // Compute base per-card durations for phases.
      float earlyPerCardTotal = earlyBatchCount > 0 ? earlyPhaseDuration / earlyBatchCount : 0f;
      float laterPhaseDuration = movePhaseDuration - earlyPhaseDuration;
      if (laterPhaseDuration < 0f) laterPhaseDuration = 0f;
      float laterPerCardTotal = laterBatchCount > 0 ? laterPhaseDuration / laterBatchCount : 0f;

      // Adjust if later cards get too little time; borrow from shuffle wiggle if necessary.
      if (laterBatchCount > 0 && laterPerCardTotal / 2f < minLegDuration)
      {
        // Required later duration to hit minLegDuration with no pause.
        float requiredLaterDuration = laterBatchCount * (minLegDuration * 2f);
        float deficit = requiredLaterDuration - laterPhaseDuration;
        if (deficit > 0f)
        {
          // First try trimming shuffle time.
            float availableFromShuffle = Mathf.Max(0f, shuffleRotationDuration - 0.15f); // keep at least 0.15s
          float take = Mathf.Min(deficit, availableFromShuffle);
          shuffleRotationDuration -= take;
          laterPhaseDuration += take;
          laterPerCardTotal = laterPhaseDuration / laterBatchCount;
        }
      }

      for (int i = 0; i < sourceCards.Count; ++i)
      {
        var card = sourceCards[i];
        source.RemoveIfPresent(card);

        bool isEarly = i < earlyBatchCount;
        float perCardTotal = isEarly ? earlyPerCardTotal : laterPerCardTotal;

        // Early cards get a pause that tapers off over the first 10. Later cards have no pause.
        float pauseTaper = 0f;
        if (isEarly && earlyBatchCount > 1)
        {
          pauseTaper = 1f - (i / (float)(earlyBatchCount - 1)); // 1 for first early, 0 for last early
        }
        else if (isEarly && earlyBatchCount == 1)
        {
          pauseTaper = 1f;
        }
        float pauseDuration = isEarly ? perCardTotal * earlyPauseFraction * pauseTaper : 0f;
        float legsAvailable = perCardTotal - pauseDuration;
        if (legsAvailable < minLegDuration * 2f)
        {
          pauseDuration = Mathf.Max(0f, perCardTotal - (minLegDuration * 2f));
          legsAvailable = perCardTotal - pauseDuration;
        }
        float legDuration = legsAvailable / 2f;
        legDuration = Mathf.Max(minLegDuration, legDuration);

        Registry.SoundService.PlayDrawCardSound();

        // 1) Move to drawn position.
        yield return MoveCardToPosition(card,
          Registry.Layout.DrawnCardsPosition.transform.position,
          Registry.Layout.DrawnCardsPosition.transform.rotation,
          legDuration);

        card.GameContext = GameContext.DrawnCards;

        if (pauseDuration > 0.001f)
        {
          yield return new WaitForSeconds(pauseDuration);
        }

        // 2) Move into deck root.
        yield return MoveCardToPosition(card,
          destination.transform.position,
          destination.transform.rotation,
          legDuration);

        destination.Add(card);
        card.transform.position = destination.transform.position;
        card.transform.rotation = destination.transform.rotation;
      }

      destination.ApplyLayout();

      // 3) Shuffle rotation effect (wiggle cards around Y axis within remaining
      //    time budget)
      yield return ShuffleDeckRotation(destination, shuffleRotationDuration);
    }

    IEnumerator MoveCardToPosition(Card card, Vector3 position, Quaternion rotation, float duration)
    {
      var seq = DOTween.Sequence();
      seq.Insert(0, card.transform.DOMove(position, duration).SetEase(Ease.OutCubic));
      seq.Insert(0, card.transform.DORotateQuaternion(rotation, duration).SetEase(Ease.OutCubic));
      yield return seq.WaitForCompletion();
    }

    IEnumerator ShuffleDeckRotation(ObjectLayout deckLayout, float totalDuration)
    {
      var cards = deckLayout.Objects.OfType<Card>().ToList();
      if (cards.Count == 0 || totalDuration <= 0f)
      {
        yield break;
      }

      // Single quick wiggle: rotate to a small random Y angle then back.
      float half = totalDuration / 2f;
      var seq = DOTween.Sequence();
      foreach (var card in cards)
      {
        var startEuler = card.transform.localEulerAngles;
        float angle = Random.Range(-15f, 15f);
        var midEuler = startEuler + new Vector3(0f, angle, 0f);
        seq.Insert(0, card.transform.DOLocalRotate(midEuler, half * 0.9f).SetEase(Ease.OutCubic));
        seq.Insert(half, card.transform.DOLocalRotate(startEuler, half * 0.9f).SetEase(Ease.InCubic));
      }
      yield return seq.WaitForCompletion();
    }

    IEnumerator DrawUserCard(DrawUserCardsCommand command, int index, bool isLastCard)
    {
      var cardView = command.Cards[index];
      var card = Registry.LayoutService.GetCard(cardView.Id);
      if (card.Parent)
      {
        card.Parent.RemoveIfPresent(card);
      }

      Registry.SoundService.PlayDrawCardSound();
      var sequence = TweenUtils.Sequence("DrawUserCard");
      var moveDuration = 0.3f;
      card.SortingKey = (int)cardView.Position.SortingKey;
      card.Render(Registry, cardView, sequence);
      card.GameContext = GameContext.DrawnCards;
      sequence.Insert(
        0,
        card.transform.DOMove(
          Registry.Layout.DrawnCardsPosition.transform.position, moveDuration)
            .SetEase(Ease.OutCubic));
      sequence.Insert(
        0,
        card.transform.DORotateQuaternion(Registry.Layout.DrawnCardsPosition.transform.rotation, moveDuration));
      yield return new WaitForSeconds(moveDuration + command.PauseDuration.ToSeconds());

      var layout = Registry.LayoutService.LayoutForPosition(command.Destination);
      layout.Add(card);
      if (!isLastCard)
      {
        // Running this on the last card will conflict with LayoutService.ApplyLayout.
        layout.ApplyLayout(TweenUtils.Sequence("DrawUserCardMoveToHand"));
      }
    }

    /// <summary>
    /// Displays a large format version of the provided card in the info zoom.
    /// </summary>
    public void DisplayInfoZoom(Card card, bool forCardInHand)
    {
      if ((_currentInfoZoom && card.Id == _currentInfoZoom.Id) || _infoZoomDisabled)
      {
        return;
      }

      if (_delayedIconClearCoroutine != null)
      {
        StopCoroutine(_delayedIconClearCoroutine);
        _delayedIconClearCoroutine = null;
        ClearInfoZoomIcons();
      }

      var shouldShowOnLeft = Registry.InputService.PointerPosition().x > Screen.width / 2.0;

      if (!shouldShowOnLeft)
      {
        _hidCloseButton = Registry.Layout.Browser.SetCloseButtonVisible(false);
      }

      if (!forCardInHand)
      {
        // Cards in hand jump to a large size in-place, we don't show a copy of them.
        _currentInfoZoom = card.CloneForInfoZoom();
        if (_currentInfoZoom.SortingGroup)
        {
          _currentInfoZoom.SortingGroup.sortingLayerID = GameContext.InfoZoom.SortingLayerId();
        }

        var anchor = shouldShowOnLeft ? Registry.Layout.InfoZoomLeft : Registry.Layout.InfoZoomRight;
        _currentInfoZoom.transform.SetParent(anchor);
        _currentInfoZoom.transform.localPosition = Vector3.zero;
        _currentInfoZoom.transform.localScale = Vector3.one;
        _currentInfoZoom.transform.forward = anchor.forward;
      }

      if (card.CardView.Revealed?.InfoZoomData?.Icons is { } icons)
      {
        foreach (var icon in icons)
        {
          var targetCard = Registry.LayoutService.GetCard(icon.CardId);
          targetCard.SetInfoZoomIcon(icon);
          _cardsWithInfoZoomIcons.Add(targetCard);
        }
      }

      if (card.CardView.Revealed?.InfoZoomData?.SupplementalCardInfo is { } info)
      {
        var infoAnchor = shouldShowOnLeft ?
            Registry.Layout.SupplementalCardInfoLeft : Registry.Layout.SupplementalCardInfoRight;
        var screenPosition = Registry.Layout.MainCamera.WorldToScreenPoint(infoAnchor.position);
        var width = Mathf.Min(275f, Registry.DocumentService.ScreenPxToElementPx(Screen.width / 2.2f));

        var node = Mason.Row("InfoZoom",
          new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Width = Mason.Px(width),
            JustifyContent = shouldShowOnLeft ? FlexJustify.FlexStart : FlexJustify.FlexEnd,
            AlignItems = FlexAlign.FlexStart,
            Inset = shouldShowOnLeft ? new FlexInsets()
            {
              Left = forCardInHand ?
                  Mason.Px(8) :
                  Mason.Px(Registry.DocumentService.ScreenPxToElementPx(screenPosition.x)),
              Top = Mason.Px(Registry.DocumentService.ScreenPxToElementPx(Screen.height - screenPosition.y)),
            } : new FlexInsets()
            {
              Right = forCardInHand ?
                  Mason.Px(8) :
                  Mason.Px(Registry.DocumentService.ScreenPxToElementPx(Screen.width - screenPosition.x)),
              Top = Mason.Px(Registry.DocumentService.ScreenPxToElementPx(Screen.height - screenPosition.y)),
            },
          },
          info
        );
        Registry.DocumentService.RenderInfoZoom(node);
      }
    }

    public void ClearInfoZoom()
    {
      if (_infoZoomDisabled)
      {
        return;
      }

      Registry.DocumentService.ClearInfoZoom();
      if (_hidCloseButton)
      {
        Registry.Layout.Browser.SetCloseButtonVisible(true);
        _hidCloseButton = false;
      }

      if (Registry.IsMobileDevice && _cardsWithInfoZoomIcons.Count > 0)
      {
        if (_delayedIconClearCoroutine != null)
        {
          StopCoroutine(_delayedIconClearCoroutine);
        }

        // On mobile, we delay clearing the info zoom icons because they will
        // frequently be covered by the InfoZoom itself.
        _delayedIconClearCoroutine = StartCoroutine(DelayedClearInfoZoomIcons());
      }
      else
      {
        ClearInfoZoomIcons();
      }

      if (_currentInfoZoom)
      {
        if (_currentInfoZoom.Parent)
        {
          _currentInfoZoom.Parent.RemoveIfPresent(_currentInfoZoom);
        }
        Destroy(_currentInfoZoom.gameObject);
      }
      _currentInfoZoom = null;
    }

    IEnumerator DelayedClearInfoZoomIcons()
    {
      yield return new WaitForSeconds(0.5f);
      ClearInfoZoomIcons();
      _delayedIconClearCoroutine = null;
    }

    void ClearInfoZoomIcons()
    {
      foreach (var card in _cardsWithInfoZoomIcons)
      {
        card.SetInfoZoomIcon(null);
      }
      _cardsWithInfoZoomIcons.Clear();
    }
  }
}