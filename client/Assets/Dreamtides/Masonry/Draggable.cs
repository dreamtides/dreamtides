#nullable enable

using System.Collections.Generic;
using System.Linq;
using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine.UIElements;

namespace Dreamtides.Masonry
{
  public sealed class Draggable : VisualElement, IMasonElement
  {
    public VisualElement Self => this;
    public Registry Registry { get; set; }
    public FlexNode? Node { get; set; }
    public List<string> TargetIdentifiers { get; set; }
    public FlexNode? OverTargetIndicator { get; set; }
    public UserAction? OnDrop { get; set; }
    public long? HorizontalDragStartDistance { get; set; }
    public bool RemoveOriginal { get; set; }
    public List<string> HideIndicatorChildren { get; set; }
    public FlexNode? CustomDragIndicator { get; set; }
    public UserAction? OnDragDetected { get; set; }
    bool _firedDragDetected;

    public static void Apply(Registry registry, Draggable view, FlexNode data)
    {
      view.Registry = registry;
      view.Node = data;
      view.TargetIdentifiers = data.NodeType.DraggableNode.DropTargetIdentifiers.ToList();
      view.OverTargetIndicator = data.NodeType.DraggableNode.OverTargetIndicator;
      view.OnDrop = Mason.ToUserAction(data.NodeType.DraggableNode.OnDrop);
      view.HorizontalDragStartDistance = data.NodeType.DraggableNode.HorizontalDragStartDistance;
      view.RemoveOriginal = data.NodeType.DraggableNode.RemoveOriginal ?? false;
      view.HideIndicatorChildren = data.NodeType.DraggableNode.HideIndicatorChildren.ToList();
      view.CustomDragIndicator = data.NodeType.DraggableNode.CustomDragIndicator;
      view.OnDragDetected = Mason.ToUserAction(data.NodeType.DraggableNode.OnDragDetected);
    }

    public Draggable()
    {
      Registry = null!;
      Node = null!;
      TargetIdentifiers = new List<string>();
      RegisterCallback<MouseDownEvent>(OnMouseDown);
      HideIndicatorChildren = new List<string>();
    }

    ~Draggable()
    {
      UnregisterCallback<MouseDownEvent>(OnMouseDown);
    }

    void OnMouseDown(MouseDownEvent evt)
    {
    }
  }
}