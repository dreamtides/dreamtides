#nullable enable

using Dreamcaller.Layout;
using Dreamcaller.Services;
using Dreamcaller.Utils;
using DG.Tweening;
using TMPro;
using UnityEngine;
using Dreamcaller.Schema;

namespace Dreamcaller.Buttons
{
  public sealed class PrimaryActionButton : Displayable
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] SpriteRenderer _background = null!;
    [SerializeField] TextMeshPro _text = null!;
    [SerializeField] Material _onPressedMaterial = null!;
    [SerializeField] float _animationDuration = 0.1f;
    [SerializeField] float _moveDistance = 1f;
    [SerializeField] AudioClip _onClickSound = null!;

    Vector3 _originalPosition;
    Color _originalColor;
    Material _originalMaterial;
    Sequence? _currentAnimation;
    UserAction? _action;

    protected override void OnStart()
    {
      _originalPosition = transform.position;
      _originalColor = _background.color;
      _originalMaterial = _background.material;
    }

    public void SetView(ButtonView? view)
    {
      if (view == null)
      {
        gameObject.SetActive(false);
        _action = null;
        _text.text = "";
      }
      else
      {
        gameObject.SetActive(true);
        _text.text = view.Label;
        _action = view.Action;
      }
    }

    public override bool CanHandleMouseEvents() => true;

    public override void MouseDown()
    {
      _currentAnimation?.Kill();
      Vector3 directionFromCamera = (transform.position - _registry.Layout.MainCamera.transform.position).normalized;
      Vector3 targetPosition = transform.position + directionFromCamera * _moveDistance;
      _currentAnimation = TweenUtils.Sequence("ButtonPressAnimation");
      _currentAnimation.Insert(0, transform.DOMove(targetPosition, _animationDuration));
      _currentAnimation.Insert(0, _background.DOColor(new Color(0.8f, 0.8f, 0.8f), _animationDuration));
      _background.material = _onPressedMaterial;
    }

    public override void MouseUp(bool isSameObject)
    {
      _currentAnimation?.Kill();
      _currentAnimation = TweenUtils.Sequence("ButtonReleaseAnimation");
      _currentAnimation.Insert(0, transform.DOMove(_originalPosition, _animationDuration));
      _currentAnimation.Insert(0, _background.DOColor(_originalColor, _animationDuration));
      _currentAnimation.InsertCallback(_animationDuration, () => _background.material = _originalMaterial);
      if (isSameObject)
      {
        _registry.SoundService.Play(_onClickSound);
        _registry.ActionService.PerformAction(_action);
      }
    }
  }
}