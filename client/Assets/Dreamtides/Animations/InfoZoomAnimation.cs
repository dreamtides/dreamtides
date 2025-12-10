#nullable enable

using System.Collections;
using System.Collections.Generic;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Masonry;
using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Animations
{
  public class InfoZoomAnimation
  {
    Card? _currentInfoZoom;
    bool _hidCloseButton;
    bool _infoZoomDisabled;
    readonly List<Card> _cardsWithInfoZoomIcons = new();
    Coroutine? _delayedIconClearCoroutine;

    public void Initialize(bool infoZoomDisabled)
    {
      _infoZoomDisabled = infoZoomDisabled;
    }

    public void DisplayInfoZoom(CardAnimationService service, Card card, bool forCardInHand)
    {
      if ((_currentInfoZoom && card.Id == _currentInfoZoom.Id) || _infoZoomDisabled)
      {
        return;
      }

      if (_delayedIconClearCoroutine != null)
      {
        service.StopCoroutine(_delayedIconClearCoroutine);
        _delayedIconClearCoroutine = null;
        ClearInfoZoomIcons();
      }

      var shouldShowOnLeft = service.Registry.IsLandscape
        ? false
        : service.Registry.InputService.PointerPosition().x > Screen.width / 2.0;

      if (!shouldShowOnLeft)
      {
        _hidCloseButton = service.Registry.BattleLayout.Browser.SetCloseButtonVisible(false);
      }

      if (!forCardInHand)
      {
        // Cards in hand jump to a large size in-place, we don't show a copy of them.
        _currentInfoZoom = card.CloneForInfoZoom();
        if (_currentInfoZoom.SortingGroup)
        {
          _currentInfoZoom.SortingGroup.sortingLayerID = GameContext.InfoZoom.SortingLayerId();
        }

        var anchor = shouldShowOnLeft
          ? service.Registry.BattleLayout.InfoZoomLeft
          : service.Registry.BattleLayout.InfoZoomRight;
        _currentInfoZoom.transform.SetParent(anchor);
        _currentInfoZoom.transform.localPosition = Vector3.zero;
        _currentInfoZoom.transform.localScale = Vector3.one;
        _currentInfoZoom.transform.forward = anchor.forward;
      }

      if (card.CardView.Revealed?.InfoZoomData?.Icons is { } icons)
      {
        foreach (var icon in icons)
        {
          var targetCard = service.Registry.CardService.GetCard(icon.CardId);
          targetCard.SetInfoZoomIcon(icon);
          _cardsWithInfoZoomIcons.Add(targetCard);
        }
      }

      if (card.CardView.Revealed?.InfoZoomData?.SupplementalCardInfo is { } info)
      {
        var infoAnchor = shouldShowOnLeft
          ? service.Registry.BattleLayout.SupplementalCardInfoLeft
          : service.Registry.BattleLayout.SupplementalCardInfoRight;
        var screenPosition = service.Registry.MainCamera.WorldToScreenPoint(infoAnchor.position);
        var width = Mathf.Min(
          275f,
          service.Registry.DocumentService.ScreenPxToElementPx(Screen.width / 2.2f)
        );

        var node = Mason.Row(
          "InfoZoom",
          new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Width = Mason.Px(width),
            JustifyContent = shouldShowOnLeft ? FlexJustify.FlexStart : FlexJustify.FlexEnd,
            AlignItems = FlexAlign.FlexStart,
            Inset = shouldShowOnLeft
              ? new FlexInsets()
              {
                Left = forCardInHand
                  ? Mason.Px(8)
                  : Mason.Px(
                    service.Registry.DocumentService.ScreenPxToElementPx(screenPosition.x)
                  ),
                Top = Mason.Px(
                  service.Registry.DocumentService.ScreenPxToElementPx(
                    Screen.height - screenPosition.y
                  )
                ),
              }
              : new FlexInsets()
              {
                Right = forCardInHand
                  ? Mason.Px(8)
                  : Mason.Px(
                    service.Registry.DocumentService.ScreenPxToElementPx(
                      Screen.width - screenPosition.x
                    )
                  ),
                Top = Mason.Px(
                  service.Registry.DocumentService.ScreenPxToElementPx(
                    Screen.height - screenPosition.y
                  )
                ),
              },
          },
          info
        );
        service.Registry.DocumentService.RenderInfoZoom(node);
      }
    }

    public void ClearInfoZoom(CardAnimationService service)
    {
      if (_infoZoomDisabled)
      {
        return;
      }

      service.Registry.DocumentService.ClearInfoZoom();
      if (_hidCloseButton)
      {
        service.Registry.BattleLayout.Browser.SetCloseButtonVisible(true);
        _hidCloseButton = false;
      }

      if (service.Registry.IsMobileDevice && _cardsWithInfoZoomIcons.Count > 0)
      {
        if (_delayedIconClearCoroutine != null)
        {
          service.StopCoroutine(_delayedIconClearCoroutine);
        }

        // On mobile, we delay clearing the info zoom icons because they will
        // frequently be covered by the InfoZoom itself.
        _delayedIconClearCoroutine = service.StartCoroutine(DelayedClearInfoZoomIcons());
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
        Object.Destroy(_currentInfoZoom.gameObject);
      }
      _currentInfoZoom = null;
    }

    IEnumerator DelayedClearInfoZoomIcons()
    {
      yield return new UnityEngine.WaitForSeconds(0.5f);
      ClearInfoZoomIcons();
      _delayedIconClearCoroutine = null;
    }

    void ClearInfoZoomIcons()
    {
      foreach (var c in _cardsWithInfoZoomIcons)
      {
        c.SetInfoZoomIcon(null);
      }
      _cardsWithInfoZoomIcons.Clear();
    }
  }
}
