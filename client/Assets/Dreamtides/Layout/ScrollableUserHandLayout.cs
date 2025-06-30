#nullable enable

using Dreamtides.Services;
using UnityEngine;
using UnityEngine.UI;

namespace Dreamtides.Layout
{
  public class ScrollableUserHandLayout : StandardObjectLayout
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] float _offset;
    [SerializeField] Scrollbar _scrollbar = null!;
    [SerializeField] float _cardWidth;
    [SerializeField] Transform _leftEdge = null!;
    [SerializeField] Transform _rightEdge = null!;
    [SerializeField] bool _hideWhenOutsideWindow;

    private float _scrollAmount = 0f;
    private float _previousScrollAmount = 0f;

    void OnEnable()
    {
      _scrollbar.value = 0;
      _scrollAmount = 0;
      _previousScrollAmount = 0;
    }

    protected override void OnUpdate()
    {
      if (Objects.Count > 0)
      {
        _scrollbar.gameObject.SetActive(Objects.Count > WindowSize() &&
            !_registry.CardService.IsPointerDownOnCard);
        _scrollbar.size = (float)WindowSize() / Objects.Count;

        _scrollAmount = _scrollbar.value;

        // Only apply layout if scroll amount has changed
        if (_scrollAmount != _previousScrollAmount)
        {
          ApplyLayout();
          _previousScrollAmount = _scrollAmount;
        }
      }
      else
      {
        _scrollbar.gameObject.SetActive(false);
      }
    }

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      return new Vector3(
        transform.position.x + ScrolledOffset(index, count, _scrollAmount),
        transform.position.y,
        transform.position.z);
    }

    protected override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;

    protected override float? CalculateObjectScale(int index, int count) => transform.localScale.x;

    /// <summary>
    /// Returns the maximum number of objects that can be displayed.
    /// </summary>
    float WindowSize() => TotalWidth() / _cardWidth;

    float TotalWidth() => Mathf.Abs(RightEdge() - LeftEdge());

    float LeftEdge() => _leftEdge.position.x;

    float RightEdge() => _rightEdge.position.x;

    /// <summary>
    /// Returns the x offset for an object at a given index, accounting for scrolling
    /// </summary>
    float ScrolledOffset(int index, int count, float scrollAmount)
    {
      var maxScrollOffset = count - WindowSize();
      var effectiveIndex = index - (scrollAmount * maxScrollOffset);

      if (_hideWhenOutsideWindow && (effectiveIndex < 0 || effectiveIndex >= WindowSize()))
      {
        // In landscape we hide the cards outside of the center to prevent
        // geometry clipping issues. Should eventually figure out a
        // cleaner fix for this.
        return -9999f;
      }

      return effectiveIndex * _offset;
    }
  }
}
