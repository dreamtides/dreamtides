#nullable enable

using System.Collections;
using System.Collections.Generic;
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
      Debug.Log("DrawUserCards");
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