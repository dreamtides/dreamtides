#nullable enable

using System.Runtime.CompilerServices;
using TMPro;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Components
{
  /// <summary>
  /// Holds a set of icons to display as part of info zoom, e.g. whether a
  /// card is a target.
  /// </summary>
  public class InfoZoomIcons : MonoBehaviour
  {
    [SerializeField]
    internal TextMeshPro _top = null!;

    [SerializeField]
    internal TextMeshPro _bottom = null!;

    [SerializeField]
    internal TextMeshPro _left = null!;

    [SerializeField]
    internal TextMeshPro _right = null!;

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
