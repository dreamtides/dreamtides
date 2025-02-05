#nullable enable

using Dreamcaller.Schema;
using Dreamcaller.Services;
using UnityEngine.UIElements;

namespace Dreamcaller.Masonry
{
  public static class Mason
  {
    public static VisualElement Render(Registry registry, FlexNode node)
    {
      var element = new NodeVisualElement();
      element.Node = node;
      return element;
    }

    public static void ApplyStyle(Registry registry, VisualElement element, FlexStyle style)
    {
    }
  }

}