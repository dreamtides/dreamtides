#nullable enable

using Dreamtides.Schema;
using TMPro;
using UnityEngine;
using UnityEngine.UI;

namespace Dreamtides.Components
{
  public class CanvasCard : MonoBehaviour
  {
    [SerializeField]
    Canvas _canvas = null!;

    [SerializeField]
    RectTransform _root = null!;

    [SerializeField]
    Image _cardImage = null!;

    [SerializeField]
    Image _cardFrame = null!;

    [SerializeField]
    TextMeshProUGUI _cardName = null!;

    [SerializeField]
    TextMeshProUGUI _cardType = null!;

    [SerializeField]
    TextMeshProUGUI _rulesText = null!;

    [SerializeField]
    Image _costBackground = null!;

    [SerializeField]
    TextMeshProUGUI _costText = null!;

    [SerializeField]
    Image _sparkBackground = null!;

    [SerializeField]
    TextMeshProUGUI _sparkText = null!;

    /// <summary>
    /// Moves a card from world space to canvas space.
    /// </summary>
    ///
    /// <remarks>
    /// This function takes a Card currently in world space and its associated CanvasCard,
    /// and causes it to become a child of the target parent on the main canvas.
    ///
    /// To the maximum extent possible, this function attempts to make this
    /// transition visually seamless, preserving the exact visual appearance of
    /// the card in its current position and orientation from the perspective of
    /// the main camera. The card will however be rotated to sit flat on the
    /// canvas.
    /// </remarks>
    public static void ToCanvas(
      Camera mainCamera,
      Canvas rootCanvas,
      RectTransform targetParent,
      Card card,
      CanvasCard canvasCard
    )
    {
      var canvasCamera =
        rootCanvas.renderMode == RenderMode.ScreenSpaceCamera ? rootCanvas.worldCamera : null;
      Debug.Log(
        $"ToCanvas start card={card.gameObject.name} rootCanvas={rootCanvas.name} targetParent={targetParent.name}"
      );

      var rootRect = rootCanvas.transform as RectTransform;
      if (rootRect == null)
      {
        Debug.LogError("ToCanvas failed: root canvas has no RectTransform");
        return;
      }

      if (card._cardFrame == null && card._cardImage == null)
      {
        Debug.LogError($"ToCanvas failed: no source renderer on {card.gameObject.name}");
        return;
      }

      if (
        !TryGetCardScreenBounds(
          mainCamera,
          card,
          out float minX,
          out float maxX,
          out float minY,
          out float maxY
        )
      )
      {
        Debug.LogError("ToCanvas failed: projected screen bounds are invalid");
        return;
      }

      var screenCenter = new Vector2((minX + maxX) * 0.5f, (minY + maxY) * 0.5f);
      var blScreen = new Vector2(minX, minY);
      var trScreen = new Vector2(maxX, maxY);

      if (
        !RectTransformUtility.ScreenPointToLocalPointInRectangle(
          rootRect,
          screenCenter,
          canvasCamera,
          out Vector2 localCenter
        )
      )
      {
        Debug.LogError(
          "ToCanvas failed: could not convert center screen point to local canvas space"
        );
        return;
      }

      var blLocal = Vector2.zero;
      var trLocal = Vector2.zero;
      if (
        !RectTransformUtility.ScreenPointToLocalPointInRectangle(
          rootRect,
          blScreen,
          canvasCamera,
          out blLocal
        )
        || !RectTransformUtility.ScreenPointToLocalPointInRectangle(
          rootRect,
          trScreen,
          canvasCamera,
          out trLocal
        )
      )
      {
        Debug.LogError(
          "ToCanvas failed: could not convert corner screen points to local canvas space"
        );
        return;
      }
      var localSize = new Vector2(
        Mathf.Max(0.01f, trLocal.x - blLocal.x),
        Mathf.Max(0.01f, trLocal.y - blLocal.y)
      );

      Debug.Log(
        $"ToCanvas screenBounds=({minX:F1},{minY:F1})-({maxX:F1},{maxY:F1}) localCenter={Vector2.zero} localSize={localSize}"
      );

      if (canvasCard._canvas)
      {
        canvasCard._canvas.enabled = false;
      }

      var rt = canvasCard._root;
      rt.SetParent(rootCanvas.transform, false);
      rt.anchorMin = new Vector2(0.5f, 0.5f);
      rt.anchorMax = new Vector2(0.5f, 0.5f);
      rt.pivot = new Vector2(0.5f, 0.5f);
      rt.localRotation = Quaternion.identity;
      rt.localScale = new Vector3(0.1f, 0.1f, 1f);
      rt.anchoredPosition = Vector2.zero;
      rt.SetAsLastSibling();

      if (card._cardFront)
      {
        card._cardFront.gameObject.SetActive(false);
      }
      if (card._battlefieldCardFront)
      {
        card._battlefieldCardFront.gameObject.SetActive(false);
      }
      if (card._cardFrame)
      {
        card._cardFrame.gameObject.SetActive(false);
      }

      Debug.Log($"ToCanvas hid world renderers for {card.gameObject.name}");

      var targetCenter = Vector2.zero;
      if (
        !RectTransformUtility.ScreenPointToLocalPointInRectangle(
          targetParent,
          screenCenter,
          canvasCamera,
          out targetCenter
        )
      )
      {
        Debug.LogError("ToCanvas failed: could not convert target center to local space");
        return;
      }

      rt.SetParent(targetParent, false);
      rt.anchoredPosition = targetCenter;

      Debug.Log($"ToCanvas completed reparent to {targetParent.name} for {card.gameObject.name}");
    }

    static bool TryGetCardScreenBounds(
      Camera mainCamera,
      Card card,
      out float minX,
      out float maxX,
      out float minY,
      out float maxY
    )
    {
      minX = float.PositiveInfinity;
      maxX = float.NegativeInfinity;
      minY = float.PositiveInfinity;
      maxY = float.NegativeInfinity;

      var corners = new Vector3[8];
      var sources = new Renderer?[] { card._cardFrame, card._cardImage };
      foreach (var childRenderer in sources)
      {
        if (childRenderer == null)
        {
          continue;
        }

        var childBounds = childRenderer.bounds;
        var childCenter = childBounds.center;
        var childExtents = childBounds.extents;

        corners[0] = childCenter + new Vector3(childExtents.x, childExtents.y, childExtents.z);
        corners[1] = childCenter + new Vector3(childExtents.x, childExtents.y, -childExtents.z);
        corners[2] = childCenter + new Vector3(childExtents.x, -childExtents.y, childExtents.z);
        corners[3] = childCenter + new Vector3(childExtents.x, -childExtents.y, -childExtents.z);
        corners[4] = childCenter + new Vector3(-childExtents.x, childExtents.y, childExtents.z);
        corners[5] = childCenter + new Vector3(-childExtents.x, childExtents.y, -childExtents.z);
        corners[6] = childCenter + new Vector3(-childExtents.x, -childExtents.y, childExtents.z);
        corners[7] = childCenter + new Vector3(-childExtents.x, -childExtents.y, -childExtents.z);

        for (var i = 0; i < corners.Length; i++)
        {
          var screenPoint = mainCamera.WorldToScreenPoint(corners[i]);
          if (screenPoint.z < 0)
          {
            continue;
          }
          if (screenPoint.x < minX)
          {
            minX = screenPoint.x;
          }
          if (screenPoint.x > maxX)
          {
            maxX = screenPoint.x;
          }
          if (screenPoint.y < minY)
          {
            minY = screenPoint.y;
          }
          if (screenPoint.y > maxY)
          {
            maxY = screenPoint.y;
          }
        }
      }

      if (
        !float.IsFinite(minX)
        || !float.IsFinite(maxX)
        || !float.IsFinite(minY)
        || !float.IsFinite(maxY)
      )
      {
        return false;
      }

      return true;
    }

    public void RenderRevealedCardView(RevealedCardView revealed)
    {
      _cardName.text = revealed.Name;
      _rulesText.text = revealed.RulesText;
      _cardType.text = revealed.CardType;

      if (_costBackground)
      {
        _costBackground.gameObject.SetActive(revealed.Cost != null);
      }
      if (_costText)
      {
        _costText.text = revealed.Cost?.ToString();
      }

      if (_sparkBackground)
      {
        _sparkBackground.gameObject.SetActive(revealed.Spark != null);
      }
      if (_sparkText)
      {
        _sparkText.text = revealed.Spark?.ToString();
      }
    }
  }
}
