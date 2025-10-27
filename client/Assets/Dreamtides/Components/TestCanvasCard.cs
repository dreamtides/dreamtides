using UnityEngine;

namespace Dreamtides.Components
{
  public class TestCanvsCard : MonoBehaviour
  {
    [SerializeField]
    Card _card = null!;

    [SerializeField]
    CanvasCard _canvasCard = null!;

    [SerializeField]
    Canvas _canvas = null!;

    [SerializeField]
    RectTransform _root = null!;

    [SerializeField]
    Camera _camera = null!;

    public void OnClick()
    {
      if (_card == null || _canvasCard == null)
      {
        var go = GameObject.Find("[shop-1] Together Against the Tide");
        if (go != null)
        {
          if (_card == null)
          {
            var c = go.GetComponent<Card>();
            if (c != null)
            {
              _card = c;
            }
          }
          if (_canvasCard == null)
          {
            var cc = go.GetComponent<CanvasCard>();
            if (cc != null)
            {
              _canvasCard = cc;
            }
          }
        }
      }

      if (_card == null || _canvasCard == null)
      {
        Debug.LogError("TestCanvasCard.OnClick missing required components");
        return;
      }

      Debug.Log(
        $"TestCanvasCard.OnClick card={_card.gameObject.name} canvasCard={_canvasCard.gameObject.name} canvas={_canvas.gameObject.name} root={_root.gameObject.name} camera={_camera.gameObject.name}"
      );
      CanvasCard.ToCanvas(_camera, _canvas, _root, _card, _canvasCard);
    }
  }
}
