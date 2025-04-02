#nullable enable

using System.Collections.Generic;
using Dreamcaller.Components;
using Dreamcaller.Schema;
using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class ArrowService : Service
  {
    [SerializeField] Arrow _redArrowPrefab = null!;
    [SerializeField] Arrow _greenArrowPrefab = null!;
    [SerializeField] Arrow _blueArrowPrefab = null!;
    readonly List<Arrow> _arrows = new();

    public void HandleDisplayArrowsCommand(DisplayArrowsCommand command)
    {
      HideArrows();
      foreach (var arrow in command.Arrows)
      {
        var arrowInstance = ComponentUtils.Instantiate(ArrowForType(arrow.Color));
        var source = Registry.LayoutService.GetGameObject(arrow.Source);
        var target = Registry.LayoutService.GetGameObject(arrow.Target);
        _arrows.Add(arrowInstance);
        arrowInstance.Source = source.transform;
        arrowInstance.Target = target.transform;
      }
    }

    public void HideArrows()
    {
      foreach (var arrow in _arrows)
      {
        Destroy(arrow.gameObject);
      }

      _arrows.Clear();
    }

    Arrow ArrowForType(ArrowStyle style) => style switch
    {
      ArrowStyle.Red => _redArrowPrefab,
      ArrowStyle.Green => _greenArrowPrefab,
      ArrowStyle.Blue => _blueArrowPrefab,
      _ => throw Errors.UnknownEnumValue(style)
    };
  }
}