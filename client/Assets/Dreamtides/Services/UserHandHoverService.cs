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
      public Vector3 JumpPosition { get; set; }
      public Quaternion JumpRotation { get; set; }
      public Tween? CurrentTween { get; set; }
      public bool IsAnimatingToJump { get; set; }
      public float AnimationProgress { get; set; }
      public float LastAnimationEndTime { get; set; }
    }

    [SerializeField] float _hoverDistance = 2f;
    [SerializeField] float _animateUpDuration = 0.1f;
    [SerializeField] float _animateDownDuration = 0.3f;
    [SerializeField] float _recoveryCheckInterval = 0.1f;
    [SerializeField] float _debounceTime = 0.3f;
    bool _isActive;
    bool _isTest;
    Card? _currentHoveredCard;
    readonly Dictionary<string, CardAnimationState> _animationStates = new();
    float _lastRecoveryCheck;

    protected override void OnInitialize(TestConfiguration? testConfiguration)
    {
      _isTest = testConfiguration != null;
    }

    protected override void OnUpdate()
    {
      if (
          _isTest ||
          Registry.IsMobileDevice ||
          Registry.DocumentService.MouseOverDocumentElement() ||
          Registry.DocumentService.HasOpenPanels
        )
      {
        return;
      }

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
        PerformRecoveryCheck();
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
          if (targetPosition == null)
          {
            continue;
          }

          var screenTargetPosition = Registry.Layout.MainCamera.WorldToScreenPoint(targetPosition.Value);
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
      PerformRecoveryCheck();
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
          if (targetPosition == null)
          {
            continue;
          }

          var screenZ = Registry.Layout.MainCamera.WorldToScreenPoint(targetPosition.Value).z;
          var worldPointerPosition = Registry.InputService.WorldPointerPosition(screenZ);
          var distance = Vector3.Distance(worldPointerPosition, targetPosition.Value);

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
          var newState = CreateAnimationState(newCard);
          if (newState == null)
          {
            return;
          }
          state = newState;
          _animationStates[newCard.Id] = state;
        }

        if (Time.time - state.LastAnimationEndTime >= _debounceTime)
        {
          state.IsAnimatingToJump = true;
          AnimateCardToJump(state);
        }
      }
    }

    CardAnimationState? CreateAnimationState(Card card)
    {
      var jumpPosition = CalculateJumpPosition(card);
      if (jumpPosition == null)
      {
        return null;
      }

      return new CardAnimationState
      {
        Card = card,
        JumpPosition = jumpPosition.Value,
        JumpRotation = Quaternion.Euler(Constants.CameraXAngle, 0, 0),
        CurrentTween = null,
        IsAnimatingToJump = false,
        AnimationProgress = 0f,
        LastAnimationEndTime = 0f
      };
    }

    public Vector3? CalculateJumpPosition(Card card)
    {
      var targetPosition = Registry.Layout.UserHand.CalculateObjectPosition(card);
      if (targetPosition == null)
      {
        return null;
      }

      var horizontalPosition = Mathf.Clamp01((targetPosition.Value.x - 8f) / 14f);
      return targetPosition.Value + Vector3.Lerp(
        new Vector3(2f, 5.0f, 2f),
        new Vector3(-2f, 5.0f, 2f),
        horizontalPosition);
    }

    void AnimateCardToJump(CardAnimationState state)
    {
      if (state.CurrentTween != null)
      {
        if (state.Card.GameContext == GameContext.Hovering)
        {
          state.Card.GameContext = GameContext.Hand;
        }
        state.Card.ExcludeFromLayout = false;
        state.CurrentTween.Kill();
      }

      var jumpPosition = CalculateJumpPosition(state.Card);
      if (jumpPosition == null)
      {
        return;
      }
      state.JumpPosition = jumpPosition.Value;

      state.Card.GameContext = GameContext.Hovering;
      state.Card.ExcludeFromLayout = true;
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
          Registry.CardAnimationService.DisplayInfoZoom(state.Card, forCardInHand: true);
        });
    }

    void AnimateCardToOriginal(CardAnimationState state)
    {
      if (state.CurrentTween != null)
      {
        if (state.Card.GameContext == GameContext.Hovering)
        {
          state.Card.GameContext = GameContext.Hand;
        }
        state.Card.ExcludeFromLayout = false;
        state.CurrentTween.Kill();
      }

      var originalPosition = Registry.Layout.UserHand.CalculateObjectPosition(state.Card);
      var originalRotation = Registry.Layout.UserHand.CalculateObjectRotation(state.Card);
      if (originalPosition == null || originalRotation == null)
      {
        Registry.CardAnimationService.ClearInfoZoom();
        if (state.Card.GameContext == GameContext.Hovering)
        {
          state.Card.GameContext = GameContext.Hand;
        }
        state.Card.ExcludeFromLayout = false;
        return;
      }

      Registry.CardAnimationService.ClearInfoZoom();
      if (state.Card.GameContext == GameContext.Hovering)
      {
        state.Card.GameContext = GameContext.Hand;
      }
      state.Card.ExcludeFromLayout = false;
      state.CurrentTween = DOTween.Sequence()
        .Append(state.Card.transform.DOMove(originalPosition.Value, _animateDownDuration).SetEase(Ease.OutCubic))
        .Join(state.Card.transform.DORotateQuaternion(Quaternion.Euler(originalRotation.Value), _animateDownDuration)
          .SetEase(Ease.OutCubic)
        )
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
          state.LastAnimationEndTime = Time.time;
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
          if (state.Card.GameContext == GameContext.Hovering)
          {
            state.Card.GameContext = GameContext.Hand;
          }
          state.Card.ExcludeFromLayout = false;
          state.CurrentTween?.Kill();
          toRemove.Add(kvp.Key);
          continue;
        }

        if (state.CurrentTween == null || !state.CurrentTween.IsActive())
        {
          var currentPos = state.Card.transform.position;
          var targetPos = state.IsAnimatingToJump ?
            state.JumpPosition :
            userHand.CalculateObjectPosition(state.Card);

          if (targetPos == null)
          {
            continue;
          }

          if (Vector3.Distance(currentPos, targetPos.Value) > 0.01f)
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

      foreach (var key in toRemove)
      {
        _animationStates.Remove(key);
      }
    }

    void PerformRecoveryCheck()
    {
      if (Time.time - _lastRecoveryCheck < _recoveryCheckInterval)
      {
        return;
      }

      _lastRecoveryCheck = Time.time;

      var userHand = Registry.Layout.UserHand;
      if (userHand.Objects.Count == 0)
      {
        return;
      }

      foreach (var displayable in userHand.Objects)
      {
        if (displayable is Card card && card.GameContext == GameContext.Hovering)
        {
          var shouldBeHovering = _isActive && card == _currentHoveredCard;

          if (!shouldBeHovering)
          {
            card.GameContext = GameContext.Hand;

            if (_animationStates.TryGetValue(card.Id, out var state))
            {
              state.IsAnimatingToJump = false;
              AnimateCardToOriginal(state);
            }
            else
            {
              var targetPosition = userHand.CalculateObjectPosition(card);
              if (targetPosition == null)
              {
                card.ExcludeFromLayout = false;
                continue;
              }

              var currentPos = card.transform.position;

              if (Vector3.Distance(currentPos, targetPosition.Value) > 0.1f)
              {
                var newState = CreateAnimationState(card);
                if (newState != null)
                {
                  newState.IsAnimatingToJump = false;
                  _animationStates[card.Id] = newState;
                  AnimateCardToOriginal(newState);
                }
                else
                {
                  card.ExcludeFromLayout = false;
                }
              }
              else
              {
                card.ExcludeFromLayout = false;
              }
            }
          }
        }
      }
    }
  }
}