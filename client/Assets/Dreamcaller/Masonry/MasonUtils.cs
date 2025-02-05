#nullable enable

using Dreamcaller.Schema;

namespace Dreamcaller.Masonry
{
  public static class MasonUtils
  {
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

    public static NodeType GetNodeType(FlexNode? node)
    {
      if (node == null)
      {
        return NodeType.VisualElement;
      }

      if (node.NodeType.Text != null)
      {
        return NodeType.Text;
      }

      if (node.NodeType.ScrollViewNode != null)
      {
        return NodeType.ScrollView;
      }

      if (node.NodeType.DraggableNode != null)
      {
        return NodeType.Draggable;
      }

      if (node.NodeType.TextFieldNode != null)
      {
        return NodeType.TextField;
      }

      if (node.NodeType.SliderNode != null)
      {
        return NodeType.Slider;
      }

      return NodeType.VisualElement;
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
      result.BackgroundImageAutoSize = newStyle.BackgroundImageAutoSize ?? original.BackgroundImageAutoSize;
      result.BackgroundImageScaleMode = newStyle.BackgroundImageScaleMode ?? original.BackgroundImageScaleMode;
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
