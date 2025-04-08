#nullable enable

using System;
using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine.UIElements;

namespace Dreamtides.Masonry
{
  public static class ScrollViews
  {
    public static void Apply(Registry registry, ScrollView view, ScrollViewNode data)
    {
      view.elasticity = (float)(data.Elasticity ?? 0.1f);
      view.horizontalPageSize = (float)(data.HorizontalPageSize ?? -1);
      if (data.HorizontalScrollBar != null)
      {
        ApplyScrollerStyle(registry, view.horizontalScroller, data.HorizontalScrollBar);
      }
      view.horizontalScrollerVisibility = AdaptVisibility(data.HorizontalScrollBarVisibility);
      view.scrollDecelerationRate = (float)(data.ScrollDecelerationRate ?? 0.135f);
      view.touchScrollBehavior = data.TouchScrollBehavior switch
      {
        null => ScrollView.TouchScrollBehavior.Clamped,
        TouchScrollBehavior.Unrestricted => ScrollView.TouchScrollBehavior.Unrestricted,
        TouchScrollBehavior.Elastic => ScrollView.TouchScrollBehavior.Elastic,
        TouchScrollBehavior.Clamped => ScrollView.TouchScrollBehavior.Clamped,
        _ => throw new ArgumentOutOfRangeException()
      };
      view.verticalPageSize = (float)(data.VerticalPageSize ?? -1);
      view.verticalScrollerVisibility = AdaptVisibility(data.VerticalScrollBarVisibility);
      if (data.VerticalScrollBar != null)
      {
        ApplyScrollerStyle(registry, view.verticalScroller, data.VerticalScrollBar);
      }
      view.mouseWheelScrollSize = (float)(data.MouseWheelScrollSize ?? 1.0f);
    }

    static void ApplyScrollerStyle(Registry registry, Scroller scroller, ScrollBar data)
    {
      foreach (var child in scroller.Query<VisualElement>().Build())
      {
        MasonRenderer.ApplyStyle(registry, child, data.Style);
      }
    }

    static ScrollerVisibility AdaptVisibility(ScrollBarVisibility? visibility) =>
      visibility switch
      {
        null => ScrollerVisibility.Auto,
        ScrollBarVisibility.Auto => ScrollerVisibility.Auto,
        ScrollBarVisibility.AlwaysVisible => ScrollerVisibility.AlwaysVisible,
        ScrollBarVisibility.Hidden => ScrollerVisibility.Hidden,
        _ => throw new ArgumentOutOfRangeException(nameof(visibility), visibility, null)
      };
  }

  public sealed class NodeScrollView : ScrollView, INodeCallbacks
  {
    public VisualElement Self => this;
    readonly Lazy<Callbacks> _callbacks = new();
    public Lazy<Callbacks> Callbacks => _callbacks;
    public FlexNode? Node { get; set; }
  }
}