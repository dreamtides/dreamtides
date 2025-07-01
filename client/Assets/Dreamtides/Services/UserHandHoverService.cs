#nullable enable

using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Services
{
  /// <summary>
  /// Handles the hover state of cards in the user's hand.
  /// </summary>
  public class UserHandHoverService : Service
  {
    class CardAnimationState
    {
      public Card Card { get; set; } = null!;
      public Vector3 OriginalPosition { get; set; }
      public Quaternion OriginalRotation { get; set; }
      public Vector3 JumpPosition { get; set; }
      public Quaternion JumpRotation { get; set; }
      public Tween? CurrentTween { get; set; }
      public bool IsAnimatingToJump { get; set; }
      public float AnimationProgress { get; set; }
    }

    [SerializeField] float _hoverDistance = 2f;
    [SerializeField] float _animateUpDuration = 0.1f;
    [SerializeField] float _animateDownDuration = 0.3f;
    bool _isActive;
    Card? _currentHoveredCard;
    readonly Dictionary<string, CardAnimationState> _animationStates = new();

    protected override void OnInitialize(TestConfiguration? testConfiguration)
    {
    }

    protected override void OnUpdate()
    {
      if (Registry.InputService.InputProvider.IsPointerPressed())
      {
        if (_isActive)
        {
          UpdateAllAnimations();
        }
        return;
      }

      var shouldBeActive = ShouldActivateHover();

      if (shouldBeActive && !_isActive)
      {
        _isActive = true;
      }
      else if (!shouldBeActive && _isActive)
      {
        _isActive = false;
        AnimateAllCardsToOriginal();
        _currentHoveredCard = null;
        _animationStates.Clear();
        return;
      }

      if (!_isActive)
      {
        return;
      }

      var userHand = Registry.Layout.UserHand;
      if (userHand.Objects.Count == 0)
      {
        return;
      }

      var pointerPosition = Registry.InputService.InputProvider.PointerPosition();
      Card? closestCard = null;
      var closestDistance = float.MaxValue;

      foreach (var displayable in userHand.Objects)
      {
        if (displayable is Card card)
        {
          var targetPosition = userHand.CalculateObjectPosition(card);
          var screenTargetPosition = Registry.Layout.MainCamera.WorldToScreenPoint(targetPosition);
          var distance = Vector2.Distance(pointerPosition, new Vector2(screenTargetPosition.x, screenTargetPosition.y));

          if (distance < closestDistance)
          {
            closestDistance = distance;
            closestCard = card;
          }
        }
      }

      if (closestCard != _currentHoveredCard)
      {
        TransitionToNewCard(closestCard);
      }

      UpdateAllAnimations();
    }

    bool ShouldActivateHover()
    {
      var userHand = Registry.Layout.UserHand;
      if (userHand.Objects.Count == 0)
      {
        return false;
      }

      foreach (var displayable in userHand.Objects)
      {
        if (displayable is Card card)
        {
          var targetPosition = userHand.CalculateObjectPosition(card);
          var screenZ = Registry.Layout.MainCamera.WorldToScreenPoint(targetPosition).z;
          var worldPointerPosition = Registry.InputService.WorldPointerPosition(screenZ);
          var distance = Vector3.Distance(worldPointerPosition, targetPosition);

          if (distance <= _hoverDistance)
          {
            return true;
          }
        }
      }

      return false;
    }

    void TransitionToNewCard(Card? newCard)
    {
      if (_currentHoveredCard != null && _animationStates.TryGetValue(_currentHoveredCard.Id, out var previousState))
      {
        previousState.IsAnimatingToJump = false;
        AnimateCardToOriginal(previousState);
      }

      _currentHoveredCard = newCard;

      if (newCard != null)
      {
        if (!_animationStates.TryGetValue(newCard.Id, out var state))
        {
          state = CreateAnimationState(newCard);
          _animationStates[newCard.Id] = state;
        }

        state.IsAnimatingToJump = true;
        AnimateCardToJump(state);
      }
    }

    CardAnimationState CreateAnimationState(Card card)
    {
      var targetPosition = Registry.Layout.UserHand.CalculateObjectPosition(card);
      var jumpPosition = CalculateJumpPosition(card, targetPosition);

      return new CardAnimationState
      {
        Card = card,
        OriginalPosition = targetPosition,
        OriginalRotation = Quaternion.Euler(Constants.CameraXAngle, 0, 0),
        JumpPosition = jumpPosition,
        JumpRotation = Quaternion.Euler(Constants.CameraXAngle, 0, 0),
        CurrentTween = null,
        IsAnimatingToJump = false,
        AnimationProgress = 0f
      };
    }

    Vector3 CalculateJumpPosition(Card card, Vector3 targetPosition)
    {
      if (Registry.IsLandscape)
      {
        var horizontalPosition = Mathf.Clamp01((targetPosition.x - 8f) / 14f);
        return targetPosition + Vector3.Lerp(
          new Vector3(2f, 6f, 2.5f),
          new Vector3(-2f, 6f, 2.5f),
          horizontalPosition);
      }
      else
      {
        var screenZ = Registry.Layout.MainCamera.WorldToScreenPoint(targetPosition).z;
        var worldPosition = Registry.InputService.WorldPointerPosition(screenZ);
        var offset = targetPosition - worldPosition;
        var result = targetPosition + new Vector3(0, 3, Mathf.Max(1.75f, 3.25f - offset.z));
        result.x = Mathf.Clamp(result.x,
            Registry.Layout.InfoZoomLeft.position.x,
            Registry.Layout.InfoZoomRight.position.x);
        result.y = Mathf.Clamp(result.y, 20f, 25f);
        result.z = Mathf.Clamp(result.z, -25f, -20f);
        return result;
      }
    }

    void AnimateCardToJump(CardAnimationState state)
    {
      state.CurrentTween?.Kill();

      state.CurrentTween = DOTween.Sequence()
        .Append(state.Card.transform.DOMove(state.JumpPosition, _animateUpDuration).SetEase(Ease.OutCubic))
        .Join(state.Card.transform.DORotateQuaternion(state.JumpRotation, _animateUpDuration).SetEase(Ease.OutCubic))
        .OnUpdate(() =>
        {
          if (state.IsAnimatingToJump)
          {
            state.AnimationProgress = Mathf.Clamp01(state.AnimationProgress + Time.deltaTime / _animateUpDuration);
          }
        })
        .OnComplete(() =>
        {
          state.AnimationProgress = 1f;
          state.CurrentTween = null;
        });
    }

    void AnimateCardToOriginal(CardAnimationState state)
    {
      state.CurrentTween?.Kill();

      state.CurrentTween = DOTween.Sequence()
        .Append(state.Card.transform.DOMove(state.OriginalPosition, _animateDownDuration).SetEase(Ease.OutCubic))
        .Join(state.Card.transform.DORotateQuaternion(state.OriginalRotation, _animateDownDuration).SetEase(Ease.OutCubic))
        .OnUpdate(() =>
        {
          if (!state.IsAnimatingToJump)
          {
            state.AnimationProgress = Mathf.Clamp01(state.AnimationProgress - Time.deltaTime / _animateDownDuration);
          }
        })
        .OnComplete(() =>
        {
          state.AnimationProgress = 0f;
          state.CurrentTween = null;
          if (!state.IsAnimatingToJump)
          {
            _animationStates.Remove(state.Card.Id);
          }
        });
    }

    void AnimateAllCardsToOriginal()
    {
      foreach (var state in _animationStates.Values.ToList())
      {
        state.IsAnimatingToJump = false;
        AnimateCardToOriginal(state);
      }
    }

    void UpdateAllAnimations()
    {
      var userHand = Registry.Layout.UserHand;
      var toRemove = new List<string>();

      foreach (var kvp in _animationStates)
      {
        var state = kvp.Value;

        if (!userHand.Objects.Contains(state.Card))
        {
          state.CurrentTween?.Kill();
          toRemove.Add(kvp.Key);
          continue;
        }

        var newTargetPosition = userHand.CalculateObjectPosition(state.Card);
        if (Vector3.Distance(newTargetPosition, state.OriginalPosition) > 0.01f)
        {
          state.OriginalPosition = newTargetPosition;
          state.JumpPosition = CalculateJumpPosition(state.Card, newTargetPosition);

          if (state.CurrentTween == null || !state.CurrentTween.IsActive())
          {
            var currentPos = state.Card.transform.position;
            var targetPos = state.IsAnimatingToJump ? state.JumpPosition : state.OriginalPosition;

            if (Vector3.Distance(currentPos, targetPos) > 0.01f)
            {
              if (state.IsAnimatingToJump)
              {
                AnimateCardToJump(state);
              }
              else
              {
                AnimateCardToOriginal(state);
              }
            }
          }
        }
      }

      foreach (var key in toRemove)
      {
        _animationStates.Remove(key);
      }
    }
  }
}