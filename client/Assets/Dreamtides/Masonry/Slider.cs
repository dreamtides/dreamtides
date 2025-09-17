#nullable enable

using System;
using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine;
using UnityEngine.UIElements;

namespace Dreamtides.Masonry
{
  public static class Sliders
  {
    public static void Apply(Registry registry, Slider view, SliderNode data)
    {
      view.value = string.IsNullOrEmpty(data.PreferenceKey)
        ? (float)(data.InitialValue ?? 0)
        : PlayerPrefs.GetFloat(data.PreferenceKey);

      view.label = data.Label;

      view.direction = data.Direction switch
      {
        Schema.SliderDirection.Horizontal => UnityEngine.UIElements.SliderDirection.Horizontal,
        Schema.SliderDirection.Vertical => UnityEngine.UIElements.SliderDirection.Vertical,
        _ => UnityEngine.UIElements.SliderDirection.Horizontal,
      };

      view.highValue = (float)(data.HighValue ?? 0);
      view.lowValue = (float)(data.LowValue ?? 0);

      view.inverted = data.Inverted ?? false;
      view.pageSize = (float)(data.PageSize ?? 0);
      view.showInputField = data.ShowInputField ?? false;

      if (string.IsNullOrEmpty(data.PreferenceKey))
      {
        ((INodeCallbacks)view).SetCallback(Callbacks.Event.Change, null);
      }
      else
      {
        ((INodeCallbacks)view).SetCallback(
          Callbacks.Event.Change,
          () =>
          {
            PlayerPrefs.SetFloat(data.PreferenceKey, view.value);
            registry.SettingsService.SyncPreferences();
          }
        );
      }

      MasonRenderer.ApplyStyle(registry, view.labelElement, data.LabelStyle);
      MasonRenderer.ApplyStyle(
        registry,
        view.Query(className: BaseSlider<float>.dragContainerUssClassName),
        data.DragContainerStyle
      );
      MasonRenderer.ApplyStyle(
        registry,
        view.Query(className: BaseSlider<float>.trackerUssClassName),
        data.TrackerStyle
      );
      MasonRenderer.ApplyStyle(
        registry,
        view.Query(className: BaseSlider<float>.draggerUssClassName),
        data.DraggerStyle
      );
      MasonRenderer.ApplyStyle(
        registry,
        view.Query(className: BaseSlider<float>.draggerBorderUssClassName),
        data.DraggerBorderStyle
      );
    }
  }

  public sealed class NodeSlider : Slider, INodeCallbacks
  {
    public VisualElement Self => this;
    readonly Lazy<Callbacks> _callbacks = new();
    public Lazy<Callbacks> Callbacks => _callbacks;
    public FlexNode? Node { get; set; }
  }
}
