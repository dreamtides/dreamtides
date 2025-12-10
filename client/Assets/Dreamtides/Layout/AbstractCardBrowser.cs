#nullable enable

using System.Runtime.CompilerServices;
using DG.Tweening;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public abstract class AbstractCardBrowser : StandardObjectLayout
  {
    [SerializeField]
    protected internal float _cardWidth = 2.5f;

    [SerializeField]
    protected internal Transform _leftEdge = null!;

    [SerializeField]
    protected internal Transform _rightEdge = null!;

    [SerializeField]
    protected internal bool _isOpen = false;

    public bool IsOpen => _isOpen;

    public virtual void Show(Registry registry, Sequence? sequence)
    {
      void OnShow()
      {
        _isOpen = true;
        OnShowComplete();
      }

      if (!_isOpen)
      {
        OnShowStart();
        registry.Layout.BrowserBackground.Show(0.75f, sequence);
        if (sequence != null)
        {
          sequence.AppendCallback(OnShow);
        }
        else
        {
          OnShow();
        }
      }
    }

    public virtual void Hide(Registry registry, Sequence? sequence)
    {
      void OnHidden()
      {
        _isOpen = false;
        OnHideComplete();
      }

      if (_isOpen)
      {
        OnHideStart();
        registry.Layout.BrowserBackground.Hide(sequence);
        if (sequence != null)
        {
          sequence.AppendCallback(OnHidden);
        }
        else
        {
          OnHidden();
        }
      }
    }

    protected virtual void OnShowStart() { }

    protected virtual void OnShowComplete() { }

    protected virtual void OnHideStart() { }

    protected virtual void OnHideComplete() { }

    protected float TotalWidth() => Vector3.Distance(_leftEdge.position, _rightEdge.position);

    protected float LeftEdge() => _leftEdge.position.x;

    protected float RightEdge() => _rightEdge.position.x;

    protected Vector3 InterpolateEdgePosition(float normalizedPosition) =>
      Vector3.Lerp(_leftEdge.position, _rightEdge.position, normalizedPosition);

    protected virtual void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      if (_leftEdge)
      {
        Gizmos.DrawSphere(_leftEdge.position, radius: 1);
      }
      if (_rightEdge)
      {
        Gizmos.DrawSphere(_rightEdge.position, radius: 1);
      }
    }
  }
}
