#nullable enable

using DG.Tweening;
using Dreamcaller.Services;
using UnityEngine;
using UnityEngine.UI;

namespace Dreamcaller.Layout
{
  public class CardOrderSelector : StandardObjectLayout
  {
    [SerializeField] float _cardWidth = 2.5f;
    [SerializeField] Transform _leftEdge = null!;
    [SerializeField] Transform _rightEdge = null!;
    [SerializeField] float _initialSpacing = 0.5f;
    [SerializeField] bool _zAxis = false;
    [SerializeField] bool _isOpen = false;

    public bool IsOpen => _isOpen;

    protected override Vector3 CalculateObjectPosition(int index, int count)
    {
      var offset = LinearObjectLayout.CalculateOffset(TotalWidth(), _initialSpacing, _cardWidth, index, count);
      return transform.position + new Vector3(offset, 0, 0);
    }

    protected override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;


    public void Show(Registry registry, Sequence? sequence)
    {
      void OnShow()
      {
        _isOpen = true;
      }

      if (!_isOpen)
      {
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

    public void Hide(Registry registry, Sequence? sequence)
    {
      void onHidden()
      {
        _isOpen = false;
      }

      if (_isOpen)
      {
        registry.Layout.BackgroundOverlay.Hide(sequence);
        if (sequence != null)
        {
          sequence.AppendCallback(onHidden);
        }
        else
        {
          onHidden();
        }
      }
    }


    float TotalWidth() => Mathf.Abs(RightEdge() - LeftEdge());

    float LeftEdge() => _zAxis ? _leftEdge.position.z : _leftEdge.position.x;

    float RightEdge() => _zAxis ? _rightEdge.position.z : _rightEdge.position.x;

    void OnDrawGizmosSelected()
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