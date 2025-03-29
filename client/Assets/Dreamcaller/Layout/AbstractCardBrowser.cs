#nullable enable

using DG.Tweening;
using Dreamcaller.Services;
using UnityEngine;

namespace Dreamcaller.Layout
{
  public abstract class AbstractCardBrowser : StandardObjectLayout
  {
    [SerializeField] protected float _cardWidth = 2.5f;
    [SerializeField] protected Transform _leftEdge = null!;
    [SerializeField] protected Transform _rightEdge = null!;
    [SerializeField] protected bool _zAxis = false;
    [SerializeField] protected bool _isOpen = false;

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
        registry.Layout.BackgroundOverlay.Show(BackgroundOverlay.DisplayOver.Battlefield, 0.75f, sequence);
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
        registry.Layout.BackgroundOverlay.Hide(sequence);
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

    protected float TotalWidth() => Mathf.Abs(RightEdge() - LeftEdge());

    protected float LeftEdge() => _zAxis ? _leftEdge.position.z : _leftEdge.position.x;

    protected float RightEdge() => _zAxis ? _rightEdge.position.z : _rightEdge.position.x;

    protected float GetAxisPosition(Transform t) => _zAxis ? t.position.z : t.position.x;

    protected Vector3 ToAxisPosition(float distance) =>
         _zAxis ? new Vector3(0, 0, -distance) : new Vector3(distance, 0, 0);

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