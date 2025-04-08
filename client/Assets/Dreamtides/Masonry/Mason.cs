#nullable enable

using System;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Schema;
using UnityEngine;

namespace Dreamtides.Masonry
{
  public static class Mason
  {
    public static Dimension Px(float value) => new()
    {
      Unit = DimensionUnit.Pixels,
      Value = value
    };

    public static Dimension Percent(float value) => new()
    {
      Unit = DimensionUnit.Percentage,
      Value = value
    };

    public static DimensionGroup PositionDip(float left, float top) => GroupPx(top, 0, 0, left);

    public static DimensionGroup AllPx(float all) => GroupPx(all, all, all, all);

    public static DimensionGroup LeftRightPx(float leftRight) => GroupPx(0, leftRight, 0, leftRight);

    public static DimensionGroup TopBottomPx(float topBottom) => GroupPx(topBottom, 0, topBottom, 0);

    public static DimensionGroup TopDip(float top) => GroupPx(top, 0, 0, 0);

    public static DimensionGroup RightDip(float right) => GroupPx(0, right, 0, 0);

    public static DimensionGroup BottomDip(float bottom) => GroupPx(0, 0, bottom, 0);

    public static DimensionGroup LeftDip(float left) => GroupPx(0, 0, 0, left);

    public static DimensionGroup GroupPx(float top, float right, float bottom, float left) => new()
    {
      Top = Px(top),
      Right = Px(right),
      Bottom = Px(bottom),
      Left = Px(left)
    };

    public static DisplayColor MakeColor(string hexString)
    {
      if (ColorUtility.TryParseHtmlString(hexString, out var color))
      {
        return MakeColor(color);
      }
      else
      {
        throw new ArgumentException($"Invalid color: {hexString}");
      }
    }

    public static DisplayColor MakeColor(Color color, float? setAlpha = null) => new()
    {
      Red = color.r,
      Green = color.g,
      Blue = color.b,
      Alpha = setAlpha ?? color.a
    };

    public static BorderColor AllBordersColor(Color color) => new()
    {
      Top = MakeColor(color),
      Right = MakeColor(color),
      Bottom = MakeColor(color),
      Left = MakeColor(color)
    };

    public static BorderWidth AllBordersWidth(float width) => new()
    {
      Top = width,
      Right = width,
      Bottom = width,
      Left = width
    };

    public static BorderRadius AllBordersRadiusDip(float radius) => new()
    {
      TopLeft = Px(radius),
      TopRight = Px(radius),
      BottomRight = Px(radius),
      BottomLeft = Px(radius)
    };

    public static SpriteAddress Sprite(string address) => new()
    {
      Sprite = address
    };

    public static FontAddress Font(string address) => new()
    {
      Font = address
    };

    public static FlexNode Row(string name, FlexStyle? style, IEnumerable<FlexNode?> children) =>
      Row(name, style, children.ToArray());

    public static FlexNode Row(string name, FlexStyle? style = null, params FlexNode?[] children) =>
      Row(name, style, handlers: null, children);

    public static FlexNode Row(
      string name,
      FlexStyle? style = null,
      EventHandlers? handlers = null,
      params FlexNode?[] children)
    {
      style ??= new FlexStyle();
      style.FlexDirection = FlexDirection.Row;
      return MakeFlexbox(name, style, handlers, children);
    }

    public static FlexNode Column(string name, FlexStyle? style, IEnumerable<FlexNode?> children) =>
      Column(name, style, children.ToArray());

    public static FlexNode Column(string name, FlexStyle? style = null, params FlexNode?[] children) =>
      Column(name, style, handlers: null, children);

    public static FlexNode Column(
      string name,
      FlexStyle? style = null,
      EventHandlers? handlers = null,
      params FlexNode?[] children)
    {
      style ??= new FlexStyle();
      style.FlexDirection = FlexDirection.Column;
      return MakeFlexbox(name, style, handlers, children);
    }

    public static FlexNode? WithStyle(FlexNode? input, Action<FlexStyle> styleFn)
    {
      if (input != null)
      {
        styleFn(input.Style);
      }

      return input;
    }

    public static FlexNode Text(string label, FlexStyle style) => new()
    {
      NodeType = new NodeType
      {
        Text = new Text
        {
          Label = label,
        }
      },
      Style = style,
    };

    public static FlexScale Scale(float amount) => Scale(amount, amount);

    public static FlexScale Scale(float x, float y) => new()
    {
      Amount = new FlexVector3
      {
        X = x,
        Y = y,
        Z = 0
      }
    };

    public static FlexRotate Rotate(float degrees) => new()
    {
      Degrees = degrees
    };

    public static FlexTranslate TranslateDip(float x, float y, float z = 0) => new()
    {
      X = Px(x),
      Y = Px(y),
      Z = z
    };

    public static FlexTranslate TranslatePercent(float x, float y, float z = 0) => new()
    {
      X = Percent(x),
      Y = Percent(y),
      Z = z
    };


    public static Milliseconds DurationMs(uint ms) => new()
    {
      MillisecondsValue = ms
    };

    public static ImageSlice ImageSlice(uint slice) => ImageSlice(slice, slice);

    public static ImageSlice ImageSlice(uint topBottom, uint rightLeft) =>
      ImageSlice(topBottom, rightLeft, topBottom, rightLeft);

    public static ImageSlice ImageSlice(uint top, uint right, uint bottom, uint left) => new()
    {
      Top = top,
      Right = right,
      Bottom = bottom,
      Left = left
    };

    static FlexNode MakeFlexbox(string name, FlexStyle style, EventHandlers? handlers, params FlexNode?[] children)
    {
      var result = new FlexNode
      {
        Style = style,
        EventHandlers = handlers,
        Name = name,
        Children = new()
      };
      result.Children.AddRange(children.Where(child => child != null));
      return result;
    }

    public static UserAction? ToUserAction(OnClickClass? onClick)
    {
      if (onClick?.BattleAction != null)
      {
        return new UserAction { BattleAction = onClick.BattleAction };
      }

      if (onClick?.DebugAction != null)
      {
        return new UserAction { DebugAction = onClick.DebugAction };
      }

      return null;
    }

    public static NodeTypeTag GetNodeTypeTag(FlexNode? node)
    {
      if (node == null)
      {
        return NodeTypeTag.VisualElement;
      }

      if (node.NodeType?.Text != null)
      {
        return NodeTypeTag.Text;
      }

      if (node.NodeType?.ScrollViewNode != null)
      {
        return NodeTypeTag.ScrollView;
      }

      if (node.NodeType?.DraggableNode != null)
      {
        return NodeTypeTag.Draggable;
      }

      if (node.NodeType?.TextFieldNode != null)
      {
        return NodeTypeTag.TextField;
      }

      if (node.NodeType?.SliderNode != null)
      {
        return NodeTypeTag.Slider;
      }

      return NodeTypeTag.VisualElement;
    }

    /// <summary>
    /// Merges two FlexStyle objects, preferring values set in 'newStyle'.
    /// </summary>
    public static FlexStyle MergeStyles(FlexStyle original, FlexStyle newStyle)
    {
      var result = new FlexStyle();
      result.AlignContent = newStyle.AlignContent ?? original.AlignContent;
      result.AlignItems = newStyle.AlignItems ?? original.AlignItems;
      result.AlignSelf = newStyle.AlignSelf ?? original.AlignSelf;
      result.BackgroundColor = newStyle.BackgroundColor ?? original.BackgroundColor;
      result.BackgroundImage = newStyle.BackgroundImage ?? original.BackgroundImage;
      result.BackgroundImageTintColor = newStyle.BackgroundImageTintColor ?? original.BackgroundImageTintColor;
      result.BorderColor = newStyle.BorderColor ?? original.BorderColor;
      result.BorderRadius = newStyle.BorderRadius ?? original.BorderRadius;
      result.BorderWidth = newStyle.BorderWidth ?? original.BorderWidth;
      result.Color = newStyle.Color ?? original.Color;
      result.Display = newStyle.Display ?? original.Display;
      result.FlexBasis = newStyle.FlexBasis ?? original.FlexBasis;
      result.FlexDirection = newStyle.FlexDirection ?? original.FlexDirection;
      result.FlexGrow = newStyle.FlexGrow ?? original.FlexGrow;
      result.FlexShrink = newStyle.FlexShrink ?? original.FlexShrink;
      result.Font = newStyle.Font ?? original.Font;
      result.FontSize = newStyle.FontSize ?? original.FontSize;
      result.FontStyle = newStyle.FontStyle ?? original.FontStyle;
      result.Height = newStyle.Height ?? original.Height;
      result.ImageSlice = newStyle.ImageSlice ?? original.ImageSlice;
      result.Inset = newStyle.Inset ?? original.Inset;
      result.JustifyContent = newStyle.JustifyContent ?? original.JustifyContent;
      result.LetterSpacing = newStyle.LetterSpacing ?? original.LetterSpacing;
      result.Margin = newStyle.Margin ?? original.Margin;
      result.MaxHeight = newStyle.MaxHeight ?? original.MaxHeight;
      result.MaxWidth = newStyle.MaxWidth ?? original.MaxWidth;
      result.MinHeight = newStyle.MinHeight ?? original.MinHeight;
      result.MinWidth = newStyle.MinWidth ?? original.MinWidth;
      result.Opacity = newStyle.Opacity ?? original.Opacity;
      result.Overflow = newStyle.Overflow ?? original.Overflow;
      result.OverflowClipBox = newStyle.OverflowClipBox ?? original.OverflowClipBox;
      result.Padding = newStyle.Padding ?? original.Padding;
      result.ParagraphSpacing = newStyle.ParagraphSpacing ?? original.ParagraphSpacing;
      result.PickingMode = newStyle.PickingMode ?? original.PickingMode;
      result.Position = newStyle.Position ?? original.Position;
      result.Rotate = newStyle.Rotate ?? original.Rotate;
      result.Scale = newStyle.Scale ?? original.Scale;
      result.TextAlign = newStyle.TextAlign ?? original.TextAlign;
      result.TextOutlineColor = newStyle.TextOutlineColor ?? original.TextOutlineColor;
      result.TextOutlineWidth = newStyle.TextOutlineWidth ?? original.TextOutlineWidth;
      result.TextOverflow = newStyle.TextOverflow ?? original.TextOverflow;
      result.TextOverflowPosition = newStyle.TextOverflowPosition ?? original.TextOverflowPosition;
      result.TextShadow = newStyle.TextShadow ?? original.TextShadow;
      result.TransformOrigin = newStyle.TransformOrigin ?? original.TransformOrigin;
      result.TransitionDelays = newStyle.TransitionDelays ?? original.TransitionDelays;
      result.TransitionDurations = newStyle.TransitionDurations ?? original.TransitionDurations;
      result.TransitionEasingModes = newStyle.TransitionEasingModes ?? original.TransitionEasingModes;
      result.TransitionProperties = newStyle.TransitionProperties ?? original.TransitionProperties;
      result.Translate = newStyle.Translate ?? original.Translate;
      result.Visibility = newStyle.Visibility ?? original.Visibility;
      result.WhiteSpace = newStyle.WhiteSpace ?? original.WhiteSpace;
      result.Width = newStyle.Width ?? original.Width;
      result.WordSpacing = newStyle.WordSpacing ?? original.WordSpacing;
      result.Wrap = newStyle.Wrap ?? original.Wrap;
      return result;
    }
  }
}
