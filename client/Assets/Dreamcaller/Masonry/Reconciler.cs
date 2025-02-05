using Dreamcaller.Schema;
using Dreamcaller.Services;
using UnityEngine.UIElements;

#nullable enable

namespace Dreamcaller.Masonry
{
  public static class Reconciler
  {
    /// <summary>
    /// Runs the tree diff algorithm, updating the Visual Element hierarchy to match the new node state.
    /// </summary>
    /// <para>
    /// This algorithm handles two cases: it generates a new VisualElement hierarchy from a Node, and it mutates
    /// a previously-generated VisualElement hierarchy to match a new Node.
    /// </para>
    /// <param name="registry">Service registry for asset fetching during rendering</param>
    /// <param name="node">The node to render</param>
    /// <param name="previousElement">Optionally, a previously-rendered VisualElement which should be updated to match
    /// the new Node state</param>
    /// <returns>Either a new VisualElement matching the provided node, or null if <paramref name="previousElement"/>
    /// was mutated to match the provided node instead.</returns>
    public static IMasonElement? Update(
      Registry registry,
      FlexNode node,
      IMasonElement? previousElement = null)
    {
      var nodeType = MasonUtils.GetNodeType(node);
      if (previousElement != null && (MasonUtils.GetNodeType(previousElement.Node) == nodeType))
      {
        // If node types match, reuse this node
        return UpdateWhenMatching(registry, node, previousElement);
      }
      else
      {
        // Otherwise, create a new VisualElement matching this node
        return UpdateWhenNew(registry, node);
      }
    }

    static IMasonElement? UpdateWhenMatching(
      Registry registry,
      FlexNode node,
      IMasonElement previousElement)
    {
      UpdateChildren(registry, node, previousElement.Self, previousElement.Self);
      Mason.ApplyToElement(registry, previousElement, node);
      return null;
    }

    static IMasonElement UpdateWhenNew(Registry registry, FlexNode node)
    {
      var result = Mason.CreateElement(node);
      UpdateChildren(registry, node, result.Self);
      Mason.ApplyToElement(registry, result, node);
      return result;
    }

    static bool HasInternalChildren(VisualElement? element) => element is TextField or Slider;

    static void UpdateChildren(Registry registry,
      FlexNode node,
      VisualElement addTo,
      VisualElement? previousElement = null)
    {
      if (HasInternalChildren(previousElement))
      {
        // Some Unity elements have internal child elements which should not be updated.
        return;
      }

      var count = 0;
      while (count < node.Children.Count)
      {
        var child = node.Children[count];
        if (previousElement != null && count < previousElement.childCount)
        {
          // Element exists in previous tree.
          var result = Update(
            registry,
            child,
            previousElement[count] as IMasonElement);
          if (result != null)
          {
            previousElement.RemoveAt(count);
            previousElement.Insert(count, result.Self);
          }
        }
        else
        {
          addTo.Add(UpdateWhenNew(registry, child).Self);
        }

        count++;
      }

      if (previousElement != null)
      {
        while (count < previousElement.childCount)
        {
          previousElement.RemoveAt(count);
        }

      }
    }
  }
}