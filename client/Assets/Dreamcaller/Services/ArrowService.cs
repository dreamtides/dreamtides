#nullable enable

using Dreamcaller.Components;
using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class ArrowService : Service
  {
    public enum Type
    {
      Red,
      Green,
      Blue
    }

    [SerializeField] Arrow _redArrowPrefab = null!;
    [SerializeField] Arrow _greenArrowPrefab = null!;
    [SerializeField] Arrow _blueArrowPrefab = null!;
    [SerializeField] Transform _testSource = null!;
    [SerializeField] Transform _testTarget = null!;

    [SerializeField] Arrow? _currentArrow;
    Transform? _source;
    Transform? _target;

    void Start()
    {
      ShowArrow(Type.Red, _testSource, _testTarget);
    }

    public void ShowArrow(Type type, Transform source, Transform target)
    {
      HideArrows();
      _currentArrow = ComponentUtils.Instantiate(ArrowForType(type));
      _source = source;
      _target = target;
    }

    void Update()
    {
      if (_currentArrow && _source && _target)
      {
        _currentArrow.Source = _source.position;
        _currentArrow.Target = _target.position;
      }
    }

    public void HideArrows()
    {
      if (_currentArrow)
      {
        Destroy(_currentArrow.gameObject);
        _currentArrow = null;
      }
    }

    Arrow ArrowForType(Type type) => type switch
    {
      Type.Red => _redArrowPrefab,
      Type.Green => _greenArrowPrefab,
      Type.Blue => _blueArrowPrefab,
      _ => throw Errors.UnknownEnumValue(type)
    };
  }
}