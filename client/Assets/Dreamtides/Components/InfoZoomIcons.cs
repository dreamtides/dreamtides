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
        [SerializeField] TextMeshPro _top = null!;
        [SerializeField] TextMeshPro _bottom = null!;
        [SerializeField] TextMeshPro _left = null!;
        [SerializeField] TextMeshPro _right = null!;

        public void SetText(string text)
        {
            _top.text = text;
            _bottom.text = text;
            _left.text = text;
            _right.text = text;
        }
    }
}