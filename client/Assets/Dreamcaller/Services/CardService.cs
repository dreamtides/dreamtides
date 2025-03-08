#nullable enable

using System.Collections;
using DG.Tweening;
using Dreamcaller.Components;
using Dreamcaller.Layout;
using Dreamcaller.Masonry;
using Dreamcaller.Schema;
using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class CardService : Service
  {
    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];
    Card? _currentInfoZoom;

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

    public bool IsPointerOverPlayCardArea()
    {
      var ray = Registry.Layout.MainCamera.ScreenPointToRay(Registry.InputService.PointerPosition());
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);
      for (var i = 0; i < hits; ++i)
      {
        var hit = _raycastHitsTempBuffer[i];
        if (hit.collider == Registry.Layout.PlayCardArea)
        {
          return true;
        }
      }

      return false;
    }

    /// <summary>
    /// Displays a large format version of the provided card in the info zoom.
    /// </summary>
    public void DisplayInfoZoom(Card card)
    {
      if (_currentInfoZoom && card.Id == _currentInfoZoom.Id)
      {
        return;
      }

      ClearInfoZoom();
      var shouldShowOnLeft = Registry.InputService.PointerPosition().x > Screen.width / 2.0;
      _currentInfoZoom = card.CloneForInfoZoom();

      var infoZoomLayout = shouldShowOnLeft ? Registry.Layout.InfoZoomLeft : Registry.Layout.InfoZoomRight;
      var screenPoint = TransformUtils.RectTransformToScreenSpace(infoZoomLayout).center;
      var tmp = new Vector3(screenPoint.x, screenPoint.y, Registry.IsPortrait ? 10 : 8);
      var anchor = Registry.Layout.MainCamera.ScreenToWorldPoint(tmp);
      // Offset by half the card's world space size
      var cardSizeOffset = new Vector3(shouldShowOnLeft ? 1.4f : -1.4f, 0, -1.85f);
      _currentInfoZoom.transform.position = anchor + cardSizeOffset;
      _currentInfoZoom.transform.rotation = Quaternion.Euler(Constants.CameraXAngle, Registry.IsPortrait ? 0 : 90, 0);

      if (_currentInfoZoom.CardView.Revealed?.SupplementalCardInfo is { } info)
      {
        var node = Mason.Row("InfoZoom",
          new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Height = Mason.Px(160),
            JustifyContent = shouldShowOnLeft ? FlexJustify.FlexStart : FlexJustify.FlexEnd,
            AlignItems = FlexAlign.FlexStart,
            Inset = new FlexInsets()
            {
              Bottom = Mason.Px(200),
              Left = Mason.Px(0),
              Right = Mason.Px(0),
            },
          },
          info
        );
        Registry.DocumentService.RenderInfoZoom(node);
      }
    }

    public void ClearInfoZoom()
    {
      Registry.DocumentService.ClearInfoZoom();
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