#nullable enable

using System.Collections;
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
          StartCoroutine(DrawUserCard(command, i));
          yield return new WaitForSeconds(command.StaggerInterval.ToSeconds());
        }
        else
        {
          yield return DrawUserCard(command, i);
        }
      }
    }

    IEnumerator DrawUserCard(DrawUserCardsCommand command, int index)
    {
      var cardView = command.Cards[index];
      var card = Registry.LayoutService.GetCard(cardView.Id);
      if (card.Parent)
      {
        card.Parent.RemoveIfPresent(card);
      }

      var sequence = TweenUtils.Sequence("DrawUserCard");
      var moveDuration = 0.3f;
      card.Render(Registry, cardView, sequence);
      sequence.Insert(
        0,
        card.transform.DOMove(
          Registry.Layout.DrawnCardsPosition.transform.position, moveDuration)
            .SetEase(Ease.OutCubic));
      sequence.Insert(
        0,
        card.transform.DORotateQuaternion(Registry.Layout.DrawnCardsPosition.transform.rotation, moveDuration));
      yield return new WaitForSeconds(moveDuration + command.PauseDuration.ToSeconds());

      Registry.Layout.UserHand.Add(card);
      Registry.Layout.UserHand.ApplyLayout(TweenUtils.Sequence("DrawUserCardMoveToHand"));
    }

    /// <summary>
    /// Displays a large format version of the provided card in the info zoom.
    /// </summary>
    public void DisplayInfoZoom(Card card)
    {
      if ((_currentInfoZoom && card.Id == _currentInfoZoom.Id) || _infoZoomDisabled)
      {
        return;
      }

      ClearInfoZoom();

      var shouldShowOnLeft = Registry.InputService.PointerPosition().x > Screen.width / 2.0;

      // Close button overlaps infozoom
      if (!shouldShowOnLeft)
      {
        _hidCloseButton = Registry.Layout.Browser.SetCloseButtonVisible(false);
      }

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

      if (_currentInfoZoom.CardView.Revealed?.SupplementalCardInfo is { } info)
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
              Left = Mason.Px(Registry.DocumentService.ScreenPxToElementPx(screenPosition.x)),
              Top = Mason.Px(Registry.DocumentService.ScreenPxToElementPx(Screen.height - screenPosition.y)),
            } : new FlexInsets()
            {
              Right = Mason.Px(Registry.DocumentService.ScreenPxToElementPx(Screen.width - screenPosition.x)),
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
  }
}