#nullable enable

using System;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;
using UnityEngine.UIElements;

namespace Dreamtides.Masonry
{
  public static class MasonRenderer
  {
    /// <summary>
    /// Renders the provided Node into a VisualElement, recursively rendering child nodes.
    /// </summary>
    public static IMasonElement Render(Registry registry, FlexNode node)
    {
      var element = CreateElement(node);
      ApplyToElement(registry, element, node);

      foreach (var child in node.Children)
      {
        element.Self.Add(Render(registry, child).Self);
      }

      return element;
    }

    public static IMasonElement CreateElement(FlexNode node)
    {
      IMasonElement result;
      if (node.NodeType?.Text != null)
      {
        result = new NodeLabel();
      }
      else if (node.NodeType?.ScrollViewNode != null)
      {
        result = new NodeScrollView();
      }
      else if (node.NodeType?.DraggableNode != null)
      {
        result = new Draggable();
      }
      else if (node.NodeType?.TextFieldNode != null)
      {
        result = new NodeTextField();
      }
      else if (node.NodeType?.SliderNode != null)
      {
        result = new NodeSlider();
      }
      else
      {
        result = new NodeVisualElement();
      }

      result.Node = node;
      return result;
    }

    /// <summary>Applies the configuration in a Node to an existing IMasonElement, without modifying children.</summary>
    public static void ApplyToElement(Registry registry, IMasonElement element, FlexNode node)
    {
      if (node.NodeType?.Text != null)
      {
        ApplyText((NodeLabel)element, node.NodeType.Text);
      }
      else if (node.NodeType?.ScrollViewNode != null)
      {
        ScrollViews.Apply(registry, (NodeScrollView)element, node.NodeType.ScrollViewNode);
      }
      else if (node.NodeType?.DraggableNode != null)
      {
        Draggable.Apply(registry, (Draggable)element, node);
      }
      else if (node.NodeType?.TextFieldNode != null)
      {
        TextFields.Apply(registry, (NodeTextField)element, node.NodeType.TextFieldNode);
      }
      else if (node.NodeType?.SliderNode != null)
      {
        Sliders.Apply(registry, (NodeSlider)element, node.NodeType.SliderNode);
      }

      ApplyNode(registry, node, element.Self);
    }

    static void ApplyNode(Registry registry, FlexNode node, VisualElement element)
    {
      element.name = node.Name;

      ApplyStyle(registry, element, node.Style);

      if (element is INodeCallbacks callbacks)
      {
        if (node.HoverStyle != null)
        {
          var hoverStyle = new FlexStyle();
          hoverStyle = Mason.MergeStyles(hoverStyle, node.Style);
          hoverStyle = Mason.MergeStyles(hoverStyle, node.OnAttachStyle);
          hoverStyle = Mason.MergeStyles(hoverStyle, node.HoverStyle);

          callbacks.SetCallback(Callbacks.Event.MouseEnter, () =>
          {
            if (node.EventHandlers?.OnMouseEnter != null)
            {
              registry.ActionService.PerformAction(Mason.ToUserAction(node.EventHandlers.OnMouseEnter));
            }

            ApplyStyle(registry, element, hoverStyle);
          });
          callbacks.SetCallback(Callbacks.Event.MouseLeave, () =>
          {
            if (node.EventHandlers?.OnMouseLeave != null)
            {
              registry.ActionService.PerformAction(Mason.ToUserAction(node.EventHandlers.OnMouseLeave));
            }

            var originalStyle = new FlexStyle();
            originalStyle = Mason.MergeStyles(originalStyle, node.Style);
            originalStyle = Mason.MergeStyles(originalStyle, node.OnAttachStyle);
            ApplyStyle(registry, element, originalStyle);
          });
        }
        else
        {
          SetCallback(registry, callbacks,
            Mason.ToUserAction(node.EventHandlers?.OnMouseEnter),
            Callbacks.Event.MouseEnter);
          SetCallback(registry, callbacks,
            Mason.ToUserAction(node.EventHandlers?.OnMouseLeave),
            Callbacks.Event.MouseLeave);
        }

        if (node.PressedStyle != null)
        {
          var pressedStyle = new FlexStyle();
          pressedStyle = Mason.MergeStyles(pressedStyle, node.Style);
          pressedStyle = Mason.MergeStyles(pressedStyle, node.OnAttachStyle);
          pressedStyle = Mason.MergeStyles(pressedStyle, node.PressedStyle);

          callbacks.SetCallback(Callbacks.Event.MouseDown, () =>
          {
            ApplyStyle(registry, element, pressedStyle);

            if (node.EventHandlers?.OnMouseDown is { } onMouseDown)
            {
              registry.ActionService.PerformAction(Mason.ToUserAction(onMouseDown));
            }
          });
          callbacks.SetCallback(Callbacks.Event.MouseUp, () =>
          {
            var style = new FlexStyle();
            style = Mason.MergeStyles(style, node.Style);
            style = Mason.MergeStyles(style, node.OnAttachStyle);

            if (node.HoverStyle != null)
            {
              style = new FlexStyle();
              style = Mason.MergeStyles(style, node.Style);
              style = Mason.MergeStyles(style, node.OnAttachStyle);
              style = Mason.MergeStyles(style, node.HoverStyle);
            }

            ApplyStyle(registry, element, style);

            if (node.EventHandlers?.OnMouseUp is { } onMouseUp)
            {
              registry.ActionService.PerformAction(Mason.ToUserAction(onMouseUp));
            }
          });
        }
        else
        {
          SetCallback(registry, callbacks,
            Mason.ToUserAction(node.EventHandlers?.OnMouseDown),
            Callbacks.Event.MouseDown);
          SetCallback(registry, callbacks,
            Mason.ToUserAction(node.EventHandlers?.OnMouseUp),
            Callbacks.Event.MouseUp);
        }

        if (node.OnAttachStyle != null)
        {
          var attachStyle = new FlexStyle();
          attachStyle = Mason.MergeStyles(attachStyle, node.Style);
          attachStyle = Mason.MergeStyles(attachStyle, node.OnAttachStyle);

          if (element.panel != null)
          {
            ApplyStyle(registry, element, attachStyle);
          }
          else
          {
            callbacks.SetCallback(Callbacks.Event.AttachToPanel, () =>
            {
              TweenUtils.ExecuteAfter(0.01f, () =>
              {
                ApplyStyle(registry, element, attachStyle);
              });
            });
          }
        }
        else
        {
          callbacks.SetCallback(Callbacks.Event.AttachToPanel, null);
        }

        SetCallback(registry, callbacks,
          Mason.ToUserAction(node.EventHandlers?.OnClick), Callbacks.Event.Click);
        SetCallback(registry, callbacks,
          Mason.ToUserAction(node.EventHandlers?.OnLongPress), Callbacks.Event.LongPress);
        SetCallback(registry, callbacks,
          Mason.ToUserAction(node.EventHandlers?.OnFieldChanged), Callbacks.Event.FieldChanged);

        if (node.PressedStyle != null || node.HoverStyle != null || node.EventHandlers != null)
        {
          element.pickingMode = PickingMode.Position;
        }
        else
        {
          // Ignore mouse events on non-interactive elements
          element.pickingMode = PickingMode.Ignore;
        }
      }
      else
      {
        if (node.PressedStyle != null || node.HoverStyle != null || node.EventHandlers != null)
        {
          LogUtils.LogError($"Custom element {element} cannot have interaction");
        }
      }
    }

    static void SetCallback(Registry registry, INodeCallbacks element, UserAction? action, Callbacks.Event eventType)
    {
      if (action != null)
      {
        element.SetCallback(eventType, () =>
        {
          registry.ActionService.PerformAction(action);
        });
      }
      else
      {
        element.SetCallback(eventType, null);
      }
    }

    static void ApplyText(Label label, Text text)
    {
      label.text = text.Label;
    }

    public static Color ToUnityColor(DisplayColor color) => new(
      (float)color.Red, (float)color.Green, (float)color.Blue, (float)color.Alpha);

    static StyleColor AdaptColor(DisplayColor? color) =>
      color == null ? new StyleColor(StyleKeyword.Null) : ToUnityColor(color);

    static StyleFloat AdaptFloat(double? input) => (float?)input ?? new StyleFloat(StyleKeyword.Null);

    static StyleInt AdaptInt(long? input) => (int?)input ?? new StyleInt(StyleKeyword.Null);

    public static Vector2 AdaptVector2(FlexVector2? input) =>
      input is { } v ? new Vector2((float)v.X, (float)v.Y) : Vector2.zero;

    public static Vector3 AdaptVector3(FlexVector3? input) =>
      input is { } v ? new Vector3((float)v.X, (float)v.Y, (float)v.Z) : Vector3.zero;

    static StyleLength AdaptDimension(Registry registry, Dimension? dimension) =>
      dimension is { } d ? AdaptDimensionNonNull(registry, d) : new StyleLength(StyleKeyword.Null);

    static Length AdaptDimensionNonNull(Registry registry, Dimension dimension) => dimension.Unit switch
    {
      DimensionUnit.Pixels => new Length((float)dimension.Value),
      DimensionUnit.Percentage => Length.Percent((float)dimension.Value),
      DimensionUnit.ViewportWidth => new Length(
        registry.DocumentService.ScreenPxToElementPx(
          (float)dimension.Value / 100 * Screen.safeArea.width)),
      DimensionUnit.ViewportHeight => new Length(
        registry.DocumentService.ScreenPxToElementPx(
          (float)dimension.Value / 100 * Screen.safeArea.height)),
      DimensionUnit.SafeAreaTop => new Length((float)(registry.DocumentService.GetSafeArea().Top.Value * dimension.Value)),
      DimensionUnit.SafeAreaRight => new Length((float)(registry.DocumentService.GetSafeArea().Right.Value * dimension.Value)),
      DimensionUnit.SafeAreaBottom => new Length((float)(registry.DocumentService.GetSafeArea().Bottom.Value * dimension.Value)),
      DimensionUnit.SafeAreaLeft => new Length((float)(registry.DocumentService.GetSafeArea().Left.Value * dimension.Value)),
      _ => throw Errors.UnknownEnumValue(dimension.Unit)
    };

    static StyleEnum<Align> AdaptAlign(FlexAlign? input) => input switch
    {
      FlexAlign.Auto => Align.Auto,
      FlexAlign.FlexStart => Align.FlexStart,
      FlexAlign.Center => Align.Center,
      FlexAlign.FlexEnd => Align.FlexEnd,
      FlexAlign.Stretch => Align.Stretch,
      _ => new StyleEnum<Align>(StyleKeyword.Null)
    };

    static StyleList<TResult> AdaptList<TSource, TResult>(IList<TSource>? field, Func<TSource, TResult> selector) =>
      field == null || field.Count == 0
        ? new StyleList<TResult>(StyleKeyword.Null)
        : new StyleList<TResult>(field.Select(selector).ToList());

    public static void ApplyStyle(Registry registry, VisualElement e, FlexStyle? input)
    {
      if (input == null)
      {
        return;
      }

      e.style.alignContent = AdaptAlign(input.AlignContent);
      e.style.alignItems = AdaptAlign(input.AlignItems);
      e.style.alignSelf = AdaptAlign(input.AlignSelf);
      e.style.backgroundColor = AdaptColor(input.BackgroundColor);
      e.style.borderTopColor = AdaptColor(input.BorderColor?.Top);
      e.style.borderRightColor = AdaptColor(input.BorderColor?.Right);
      e.style.borderBottomColor = AdaptColor(input.BorderColor?.Bottom);
      e.style.borderLeftColor = AdaptColor(input.BorderColor?.Left);
      e.style.borderTopLeftRadius = AdaptDimension(registry, input.BorderRadius?.TopLeft);
      e.style.borderTopRightRadius = AdaptDimension(registry, input.BorderRadius?.TopRight);
      e.style.borderBottomRightRadius = AdaptDimension(registry, input.BorderRadius?.BottomRight);
      e.style.borderBottomLeftRadius = AdaptDimension(registry, input.BorderRadius?.BottomLeft);
      e.style.borderTopWidth = AdaptFloat(input.BorderWidth?.Top);
      e.style.borderRightWidth = AdaptFloat(input.BorderWidth?.Right);
      e.style.borderBottomWidth = AdaptFloat(input.BorderWidth?.Bottom);
      e.style.borderLeftWidth = AdaptFloat(input.BorderWidth?.Left);
      e.style.top = AdaptDimension(registry, input.Inset?.Top);
      e.style.right = AdaptDimension(registry, input.Inset?.Right);
      e.style.bottom = AdaptDimension(registry, input.Inset?.Bottom);
      e.style.left = AdaptDimension(registry, input.Inset?.Left);
      e.style.color = AdaptColor(input.Color);
      e.style.display = input.Display switch
      {
        FlexDisplayStyle.Flex => DisplayStyle.Flex,
        FlexDisplayStyle.None => DisplayStyle.None,
        _ => new StyleEnum<DisplayStyle>(StyleKeyword.Null)
      };
      e.style.flexBasis = AdaptDimension(registry, input.FlexBasis);
      e.style.flexDirection = input.FlexDirection switch
      {
        Schema.FlexDirection.Column => UnityEngine.UIElements.FlexDirection.Column,
        Schema.FlexDirection.ColumnReverse => UnityEngine.UIElements.FlexDirection.ColumnReverse,
        Schema.FlexDirection.Row => UnityEngine.UIElements.FlexDirection.Row,
        Schema.FlexDirection.RowReverse => UnityEngine.UIElements.FlexDirection.RowReverse,
        _ => new StyleEnum<UnityEngine.UIElements.FlexDirection>(StyleKeyword.Null)
      };
      e.style.flexGrow = AdaptFloat(input.FlexGrow);
      e.style.flexShrink = AdaptFloat(input.FlexShrink);
      e.style.flexWrap = input.Wrap switch
      {
        FlexWrap.NoWrap => Wrap.NoWrap,
        FlexWrap.Wrap => Wrap.Wrap,
        FlexWrap.WrapReverse => Wrap.WrapReverse,
        _ => new StyleEnum<Wrap>(StyleKeyword.Null)
      };
      e.style.fontSize = AdaptDimension(registry, input.FontSize);
      e.style.height = AdaptDimension(registry, input.Height);
      e.style.justifyContent = input.JustifyContent switch
      {
        FlexJustify.FlexStart => Justify.FlexStart,
        FlexJustify.Center => Justify.Center,
        FlexJustify.FlexEnd => Justify.FlexEnd,
        FlexJustify.SpaceBetween => Justify.SpaceBetween,
        FlexJustify.SpaceAround => Justify.SpaceAround,
        _ => new StyleEnum<Justify>(StyleKeyword.Null)
      };
      e.style.letterSpacing = AdaptDimension(registry, input.LetterSpacing);
      e.style.marginTop = AdaptDimension(registry, input.Margin?.Top);
      e.style.marginRight = AdaptDimension(registry, input.Margin?.Right);
      e.style.marginBottom = AdaptDimension(registry, input.Margin?.Bottom);
      e.style.marginLeft = AdaptDimension(registry, input.Margin?.Left);
      e.style.maxHeight = AdaptDimension(registry, input.MaxHeight);
      e.style.maxWidth = AdaptDimension(registry, input.MaxWidth);
      e.style.minHeight = AdaptDimension(registry, input.MinHeight);
      e.style.minWidth = AdaptDimension(registry, input.MinWidth);
      e.style.opacity = AdaptFloat(input.Opacity);
      e.style.overflow = input.Overflow switch
      {
        FlexVisibility.Visible => Overflow.Visible,
        FlexVisibility.Hidden => Overflow.Hidden,
        _ => new StyleEnum<Overflow>(StyleKeyword.Null)
      };
      e.style.paddingTop = AdaptDimension(registry, input.Padding?.Top);
      e.style.paddingRight = AdaptDimension(registry, input.Padding?.Right);
      e.style.paddingBottom = AdaptDimension(registry, input.Padding?.Bottom);
      e.style.paddingLeft = AdaptDimension(registry, input.Padding?.Left);
      e.style.position = input.Position switch
      {
        FlexPosition.Relative => UnityEngine.UIElements.Position.Relative,
        FlexPosition.Absolute => UnityEngine.UIElements.Position.Absolute,
        _ => new StyleEnum<UnityEngine.UIElements.Position>(StyleKeyword.Null)
      };
      e.style.rotate = input.Rotate is { } r
        ? new Rotate(Angle.Degrees((float)r.Degrees))
        : new StyleRotate(StyleKeyword.Null);
      e.style.scale = input.Scale is { } s ? new Scale(AdaptVector3(s.Amount)) : new StyleScale(StyleKeyword.Null);
      e.style.textOverflow = input.TextOverflow switch
      {
        Schema.TextOverflow.Clip => UnityEngine.UIElements.TextOverflow.Clip,
        Schema.TextOverflow.Ellipsis => UnityEngine.UIElements.TextOverflow.Ellipsis,
        _ => new StyleEnum<UnityEngine.UIElements.TextOverflow>(StyleKeyword.Null)
      };
      e.style.textShadow = input.TextShadow is { } ts
        ? new UnityEngine.UIElements.TextShadow
        {
          offset = AdaptVector2(ts.Offset),
          blurRadius = (float)ts.BlurRadius,
          color = ts.Color == null ? Color.black : ToUnityColor(ts.Color)
        }
        : new StyleTextShadow(StyleKeyword.Null);
      e.style.transformOrigin = input.TransformOrigin is { } to
        ? new TransformOrigin(AdaptDimensionNonNull(registry, to.X), AdaptDimensionNonNull(registry, to.Y), (float)to.Z)
        : new StyleTransformOrigin(StyleKeyword.Null);
      e.style.transitionDelay =
        AdaptList(input.TransitionDelays, t => new TimeValue(t.MillisecondsValue, TimeUnit.Millisecond));
      e.style.transitionDuration = AdaptList(input.TransitionDurations,
        t => new TimeValue(t.MillisecondsValue, TimeUnit.Millisecond));
      e.style.transitionProperty = AdaptList(input.TransitionProperties, p => new StylePropertyName(p));
      e.style.transitionTimingFunction = AdaptList(input.TransitionEasingModes, mode => new EasingFunction(mode switch
      {
        Schema.EasingMode.Ease => UnityEngine.UIElements.EasingMode.Ease,
        Schema.EasingMode.EaseIn => UnityEngine.UIElements.EasingMode.EaseIn,
        Schema.EasingMode.EaseOut => UnityEngine.UIElements.EasingMode.EaseOut,
        Schema.EasingMode.EaseInOut => UnityEngine.UIElements.EasingMode.EaseInOut,
        Schema.EasingMode.Linear => UnityEngine.UIElements.EasingMode.Linear,
        Schema.EasingMode.EaseInSine => UnityEngine.UIElements.EasingMode.EaseInSine,
        Schema.EasingMode.EaseOutSine => UnityEngine.UIElements.EasingMode.EaseOutSine,
        Schema.EasingMode.EaseInOutSine => UnityEngine.UIElements.EasingMode.EaseInOutSine,
        Schema.EasingMode.EaseInCubic => UnityEngine.UIElements.EasingMode.EaseInCubic,
        Schema.EasingMode.EaseOutCubic => UnityEngine.UIElements.EasingMode.EaseOutCubic,
        Schema.EasingMode.EaseInOutCubic => UnityEngine.UIElements.EasingMode.EaseInOutCubic,
        Schema.EasingMode.EaseInCirc => UnityEngine.UIElements.EasingMode.EaseInCirc,
        Schema.EasingMode.EaseOutCirc => UnityEngine.UIElements.EasingMode.EaseOutCirc,
        Schema.EasingMode.EaseInOutCirc => UnityEngine.UIElements.EasingMode.EaseInOutCirc,
        Schema.EasingMode.EaseInElastic => UnityEngine.UIElements.EasingMode.EaseInElastic,
        Schema.EasingMode.EaseOutElastic => UnityEngine.UIElements.EasingMode.EaseOutElastic,
        Schema.EasingMode.EaseInOutElastic => UnityEngine.UIElements.EasingMode.EaseInOutElastic,
        Schema.EasingMode.EaseInBack => UnityEngine.UIElements.EasingMode.EaseInBack,
        Schema.EasingMode.EaseOutBack => UnityEngine.UIElements.EasingMode.EaseOutBack,
        Schema.EasingMode.EaseInOutBack => UnityEngine.UIElements.EasingMode.EaseInOutBack,
        Schema.EasingMode.EaseInBounce => UnityEngine.UIElements.EasingMode.EaseInBounce,
        Schema.EasingMode.EaseOutBounce => UnityEngine.UIElements.EasingMode.EaseOutBounce,
        Schema.EasingMode.EaseInOutBounce => UnityEngine.UIElements.EasingMode.EaseInOutBounce,
        _ => UnityEngine.UIElements.EasingMode.Ease
      }));
      e.style.translate = input.Translate is { } translate
        ? new Translate(AdaptDimensionNonNull(registry, translate.X), AdaptDimensionNonNull(registry, translate.Y),
          (float)translate.Z)
        : new StyleTranslate(StyleKeyword.Null);
      e.style.unityBackgroundImageTintColor = AdaptColor(input.BackgroundImageTintColor);
      e.style.unityFontDefinition = input.Font is { } font
        ? new StyleFontDefinition(registry.AssetService.GetFont(font))
        : new StyleFontDefinition(StyleKeyword.Null);
      e.style.unityFontStyleAndWeight = input.FontStyle switch
      {
        Schema.FontStyle.Normal => UnityEngine.FontStyle.Normal,
        Schema.FontStyle.Bold => UnityEngine.FontStyle.Bold,
        Schema.FontStyle.Italic => UnityEngine.FontStyle.Italic,
        Schema.FontStyle.BoldAndItalic => UnityEngine.FontStyle.BoldAndItalic,
        _ => new StyleEnum<UnityEngine.FontStyle>(StyleKeyword.Null)
      };
      e.style.unityOverflowClipBox = input.OverflowClipBox switch
      {
        Schema.OverflowClipBox.PaddingBox => UnityEngine.UIElements.OverflowClipBox.PaddingBox,
        Schema.OverflowClipBox.ContentBox => UnityEngine.UIElements.OverflowClipBox.ContentBox,
        _ => new StyleEnum<UnityEngine.UIElements.OverflowClipBox>(StyleKeyword.Null)
      };
      e.style.unityParagraphSpacing = AdaptDimension(registry, input.ParagraphSpacing);
      e.style.unitySliceTop = AdaptInt(input.ImageSlice?.Top);
      e.style.unitySliceRight = AdaptInt(input.ImageSlice?.Right);
      e.style.unitySliceBottom = AdaptInt(input.ImageSlice?.Bottom);
      e.style.unitySliceLeft = AdaptInt(input.ImageSlice?.Left);
      e.style.unityTextAlign = input.TextAlign switch
      {
        TextAlign.UpperLeft => TextAnchor.UpperLeft,
        TextAlign.UpperCenter => TextAnchor.UpperCenter,
        TextAlign.UpperRight => TextAnchor.UpperRight,
        TextAlign.MiddleLeft => TextAnchor.MiddleLeft,
        TextAlign.MiddleCenter => TextAnchor.MiddleCenter,
        TextAlign.MiddleRight => TextAnchor.MiddleRight,
        TextAlign.LowerLeft => TextAnchor.LowerLeft,
        TextAlign.LowerCenter => TextAnchor.LowerCenter,
        TextAlign.LowerRight => TextAnchor.LowerRight,
        _ => new StyleEnum<TextAnchor>(StyleKeyword.Null)
      };
      e.style.unityTextOutlineColor = AdaptColor(input.TextOutlineColor);
      e.style.unityTextOutlineWidth = AdaptFloat(input.TextOutlineWidth);
      e.style.unityTextOverflowPosition = input.TextOverflowPosition switch
      {
        Schema.TextOverflowPosition.End => UnityEngine.UIElements.TextOverflowPosition.End,
        Schema.TextOverflowPosition.Start => UnityEngine.UIElements.TextOverflowPosition.Start,
        Schema.TextOverflowPosition.Middle => UnityEngine.UIElements.TextOverflowPosition.Middle,
        _ => new StyleEnum<UnityEngine.UIElements.TextOverflowPosition>(StyleKeyword.Null)
      };
      e.style.visibility = input.Visibility switch
      {
        FlexVisibility.Visible => Visibility.Visible,
        FlexVisibility.Hidden => Visibility.Hidden,
        _ => new StyleEnum<Visibility>(StyleKeyword.Null)
      };
      e.style.whiteSpace = input.WhiteSpace switch
      {
        Schema.WhiteSpace.Normal => UnityEngine.UIElements.WhiteSpace.Normal,
        Schema.WhiteSpace.NoWrap => UnityEngine.UIElements.WhiteSpace.NoWrap,
        _ => new StyleEnum<UnityEngine.UIElements.WhiteSpace>(StyleKeyword.Null)
      };
      e.style.width = AdaptDimension(registry, input.Width);
      e.style.wordSpacing = AdaptDimension(registry, input.WordSpacing);

      if (input.BackgroundImage is { } bi)
      {
        var sprite = registry.AssetService.GetSprite(bi);
        var aspectRatio = sprite == null ? 0 : ((float)sprite.texture.width) / sprite.texture.height;
        e.style.backgroundImage = new StyleBackground(sprite);
      }
      else
      {
        e.style.backgroundImage = new StyleBackground(StyleKeyword.Null);
      }

      e.pickingMode = input.PickingMode switch
      {
        null => PickingMode.Position,
        FlexPickingMode.Position => PickingMode.Position,
        FlexPickingMode.Ignore => PickingMode.Ignore,
        _ => throw new ArgumentOutOfRangeException()
      };
    }
  }
}