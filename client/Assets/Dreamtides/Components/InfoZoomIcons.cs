#nullable enable

using TMPro;
using UnityEngine;

namespace Dreamtides.Components
{
  /// <summary>
  /// Holds a set of icons to display as part of info zoom, e.g. whether a
  /// card is a target.
  /// </summary>
  public class InfoZoomIcons : MonoBehaviour
  {
    [SerializeField]
    TextMeshPro _top = null!;

    [SerializeField]
    TextMeshPro _bottom = null!;

    [SerializeField]
    TextMeshPro _left = null!;

    [SerializeField]
    TextMeshPro _right = null!;

    public void SetText(string text, Color color)
    {
      _top.text = text;
      _top.color = color;
      _bottom.text = text;
      _bottom.color = color;
      _left.text = text;
      _left.color = color;
      _right.text = text;
      _right.color = color;
    }
  }
}
