#nullable enable

using System;
using System.Collections;
using System.Runtime.CompilerServices;
using DG.Tweening;
using Dreamtides.Layout;
using Dreamtides.Masonry;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using TMPro;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]
namespace Dreamtides.Components
{
  public class Card : Displayable
  {
    [SerializeField] internal Transform _cardFront = null!;
    [SerializeField] internal Transform _battlefieldCardFront = null!;
    [SerializeField] internal TextMeshPro _name = null!;
    [SerializeField] internal TextMeshPro _rulesText = null!;
    [SerializeField] internal TextMeshPro _typeText = null!;
    [SerializeField] internal Renderer _cardFrame = null!;
    [SerializeField] internal SpriteRenderer _cardImage = null!;
    [SerializeField] internal SpriteRenderer _battlefieldCardImage = null!;
    [SerializeField] internal DissolveEffect _cardImageDissolve = null!;
    [SerializeField] internal BoxCollider _cardCollider = null!;
    [SerializeField] internal Renderer _cardBack = null!;
    [SerializeField] internal Renderer _outline = null!;
    [SerializeField] internal SpriteRenderer _battlefieldOutline = null!;
    [SerializeField] internal SpriteRenderer _costBackground = null!;
    [SerializeField] internal TextMeshPro _costText = null!;
    [SerializeField] internal TextMeshPro _producedEnergyText = null!;
    [SerializeField] internal Renderer _sparkBackground = null!;
    [SerializeField] internal TextMeshPro _sparkText = null!;
    [SerializeField] internal Renderer _battlefieldSparkBackground = null!;
    [SerializeField] internal TextMeshPro _battlefieldSparkText = null!;
    [SerializeField] internal ObjectLayout? _containedObjects;
    [SerializeField] internal ObjectLayout? _stackedObjects;
    [SerializeField] internal Transform _cardTrailPosition = null!;
    [SerializeField] internal GameObject _battlefieldIconContainer = null!;
    [SerializeField] internal TextMeshPro _battlefieldIcon = null!;

    bool _isRevealed = false;
    Registry _registry = null!;
    CardView _cardView = null!;
    float _dragStartScreenZ;
    Vector3 _dragStartPosition;
    Vector3 _dragOffset;
    float _lastMouseDownTime;
    bool _isDraggingFromHand = false;
    bool _isDraggingForOrdering = false;
    bool _isDissolved = false;
    bool _draggedToClearThreshold = false;
    bool _draggedToPlayThreshold = false;
    public CardView CardView => Errors.CheckNotNull(_cardView);
    GameObject? _cardTrail;
    float _distanceDragged;
    float _hoverStartTime;
    bool _hoveringForInfoZoom;
    bool _longHoverFired;
    Vector3 _positionBeforeHover;
    Tween? _hoverMoveTween;

    public string Id => CardView.ClientId();

    public ObjectLayout ContainedObjects => Errors.CheckNotNull(_containedObjects);

    public void Render(Registry registry, CardView view, Sequence? sequence = null)
    {
      var name = view.Revealed?.Name ?? "Hidden Card";
      _registry = registry;
      _cardView = view;
      gameObject.name = $"{name} [{Id}]";

      if (view.Revealed != null)
      {
        if (_isRevealed)
        {
          RenderCardView();
        }
        else
        {
          Flip(_cardFront, _cardBack, sequence, RenderCardView);
        }
      }
      else
      {
        if (_isRevealed)
        {
          Flip(_cardBack, _cardFront, sequence, RenderCardView);
        }
        else
        {
          RenderCardView();
        }
      }
    }

    public void TurnFaceDown(Sequence? sequence = null)
    {
      Flip(_cardBack, _cardFront, sequence, () =>
      {
        _cardBack.gameObject.SetActive(true);
        _cardFront.gameObject.SetActive(false);
      });
    }

    public void TurnFaceUp(Sequence? sequence = null)
    {
      Flip(_cardFront, _cardBack, sequence, () =>
      {
        _cardBack.gameObject.SetActive(false);
        _cardFront.gameObject.SetActive(true);
      });
    }

    public IEnumerator StartDissolve(DissolveCardCommand command)
    {
      _isDissolved = true;
      ToggleActiveElements();
      yield return StartCoroutine(_cardImageDissolve.StartDissolve(_registry, command));

      if (command.Reverse)
      {
        _isDissolved = false;
        ToggleActiveElements();
      }
    }

    public void ApplyPreview(CardPreviewView preview, Color textColor)
    {
      if (preview.Cost != null)
      {
        _costText.text = preview.Cost.ToString();
        _costText.color = textColor;
      }

      if (preview.Spark != null)
      {
        _sparkText.text = preview.Spark.ToString();
        _battlefieldSparkText.text = preview.Spark.ToString();
        _sparkText.color = textColor;
        _battlefieldSparkText.color = textColor;
      }

      if (preview.BattlefieldIcon != null)
      {
        _battlefieldIconContainer.SetActive(true);
        _battlefieldIcon.text = preview.BattlefieldIcon;
        _battlefieldIcon.color = MasonRenderer.ToUnityColor(preview.BattlefieldIconColor);
      }
    }

    public void ClearPreview()
    {
      _costText.text = CardView.Revealed?.Cost?.ToString();
      _sparkText.text = CardView.Revealed?.Spark?.ToString();
      _battlefieldSparkText.text = CardView.Revealed?.Spark?.ToString();
      _costText.color = Color.white;
      _sparkText.color = Color.white;
      _battlefieldSparkText.color = Color.white;
      _battlefieldIconContainer.SetActive(false);
    }

    /// <summary>
    /// Creates a clone of the card for large display in the info zoom.
    /// </summary>
    public Card CloneForInfoZoom()
    {
      var clone = Instantiate(gameObject, transform.position, transform.rotation);
      var result = ComponentUtils.Get<Card>(clone);

      if (result._stackedObjects)
      {
        Destroy(result._stackedObjects.gameObject);
        result._stackedObjects = null;
      }

      if (result._containedObjects)
      {
        Destroy(result._containedObjects.gameObject);
        result._containedObjects = null;
      }

      result.gameObject.name = "[IZ]" + gameObject.name;
      result._cardView = CardView;
      result._outline.enabled = false;
      result._registry = _registry;
      result._isRevealed = _isRevealed;
      result.Parent = null;
      result.GameContext = GameContext.InfoZoom;
      return result;
    }

    void Update()
    {
      var outlineContext = GameContext == GameContext.Hand || GameContext == GameContext.Hovering;
      _outline.gameObject.SetActive(CanPlay() ||
          CanSelectOrder() ||
          (outlineContext && CardView.Revealed?.OutlineColor != null));
    }

    void Flip(Component faceUp, Component faceDown, Sequence? sequence, Action? onFlipped = null)
    {
      if (sequence != null)
      {
        const float duration = TweenUtils.FlipAnimationDurationSeconds / 2f;
        sequence
          .Insert(0, faceDown.transform.DOLocalRotate(new Vector3(0, 90, 0), duration))
          .InsertCallback(duration, () =>
          {
            faceUp.transform.localRotation = Quaternion.Euler(0, -90, 0);
            onFlipped?.Invoke();
          })
          .Insert(duration, faceUp.transform.DOLocalRotate(Vector3.zero, duration));
      }
      else
      {
        onFlipped?.Invoke();
      }
    }

    void RenderCardView()
    {
      if (CardView.Revealed != null)
      {
        RenderRevealedCardView(CardView.Revealed);
      }
      else
      {
        RenderHiddenCardView();
      }
    }

    void RenderRevealedCardView(RevealedCardView revealed)
    {
      _isRevealed = true;
      ToggleActiveElements();
      _name.text = revealed.Name;
      _rulesText.text = revealed.RulesText;
      _costBackground.gameObject.SetActive(revealed.Cost != null);
      _costText.text = revealed.Cost?.ToString();
      _producedEnergyText.text = revealed.Produced?.ToString();
      _sparkText.text = revealed.Spark?.ToString();
      _battlefieldSparkText.text = revealed.Spark?.ToString();
      _typeText.text = revealed.CardType;
      _cardImage.sprite = _registry.AssetService.GetSprite(revealed.Image.Address);
      _battlefieldCardImage.sprite = _registry.AssetService.GetSprite(revealed.Image.Address);
      _outline.material.SetInt("_Seed", UnityEngine.Random.Range(0, 9999));
      if (revealed.OutlineColor == null)
      {
        _outline.material.SetColor("_Color", new Color(0f, 1f, 0f));
        _outline.material.SetColor("_HiColor", new Color(0f, 1f, 0f));
        _battlefieldOutline.color = new Color(1f, 1f, 1f);
      }
      else
      {
        _outline.material.SetColor("_Color", MasonRenderer.ToUnityColor(revealed.OutlineColor));
        _outline.material.SetColor("_HiColor", MasonRenderer.ToUnityColor(revealed.OutlineColor));
        _battlefieldOutline.color = MasonRenderer.ToUnityColor(revealed.OutlineColor);
      }

      if (_cardTrail)
      {
        Destroy(_cardTrail);
      }
      if (revealed.Effects.CardTrail != null)
      {
        var trail = _registry.AssetService.GetProjectilePrefab(revealed.Effects.CardTrail);
        _cardTrail = Instantiate(trail.gameObject);
        _cardTrail.transform.SetParent(_cardTrailPosition, worldPositionStays: false);
        _cardTrail.transform.localPosition = Vector3.zero;
        _cardTrail.transform.localRotation = Quaternion.identity;
      }
    }

    void RenderHiddenCardView()
    {
      _isRevealed = false;
      ToggleActiveElements();
    }

    public override bool CanHandleMouseEvents() =>
        GameContext != GameContext.Deck &&
        GameContext != GameContext.DiscardPile &&
        GameContext != GameContext.InfoZoom;

    protected override void OnSetGameContext(GameContext oldContext, GameContext newContext)
    {
      ToggleActiveElements();
    }

    public override void MouseDown()
    {
      _lastMouseDownTime = Time.time;
      _draggedToClearThreshold = false;
      _draggedToPlayThreshold = false;
      _distanceDragged = 0;
      _registry.CardService.IsPointerDownOnCard = true;

      if (GameContext == GameContext.Hand && !_registry.CapabilitiesService.AnyBrowserOpen())
      {
        // Jump to large size when in hand
        transform.position = HandCardJumpPosition();
        transform.rotation = Quaternion.Euler(Constants.CameraXAngle, 0, 0);
      }
      else if (_registry.CapabilitiesService.CanInfoZoom(GameContext) && !_draggedToClearThreshold)
      {
        _registry.CardService.DisplayInfoZoom(this);
      }

      if (CanPlay() || CanSelectOrder())
      {
        _isDraggingFromHand = GameContext == GameContext.Hand;
        _isDraggingForOrdering = CanSelectOrder();
        _registry.SoundService.PlayCardSound();
        GameContext = GameContext.Dragging;

        if (Parent)
        {
          Parent.RemoveIfPresent(this);
        }

        _dragStartScreenZ = Camera.main.WorldToScreenPoint(gameObject.transform.position).z;
        _dragStartPosition = _registry.InputService.WorldPointerPosition(_dragStartScreenZ);
        _dragOffset = gameObject.transform.position - _dragStartPosition;
      }
      else if (GameContext == GameContext.Hand && !_registry.CapabilitiesService.AnyBrowserOpen())
      {
        GameContext = GameContext.Hovering;
      }
    }

    public override void MouseDrag()
    {
      if (!(_isDraggingFromHand || _isDraggingForOrdering))
      {
        return;
      }

      var mousePositionInStartingPlane = _registry.InputService.WorldPointerPosition(_dragStartScreenZ);
      _distanceDragged = Vector2.Distance(mousePositionInStartingPlane, _dragStartPosition);

      if (_isDraggingForOrdering || _registry.IsLandscape)
      {
        transform.position = mousePositionInStartingPlane + _dragOffset;
      }
      else
      {
        // On mobile we shrink the card down as it is dragged from hand to
        // improve visibility.
        const float playThreshold = 0.5f;
        if (_distanceDragged > playThreshold || _draggedToPlayThreshold)
        {
          _draggedToPlayThreshold = true;
          transform.position = _registry.InputService.WorldPointerPosition(20f);
        }
        else
        {
          float t = Mathf.Clamp01(_distanceDragged / playThreshold);
          Vector3 startPosition = _dragOffset + mousePositionInStartingPlane;
          Vector3 endPosition = _registry.InputService.WorldPointerPosition(20f);
          transform.position = Vector3.Lerp(startPosition, endPosition, t);
        }
      }

      if (_distanceDragged > 0.25f)
      {
        _registry.CardService.ClearInfoZoom();
        if (CardView.Revealed?.Actions.PlayEffectPreview is { } playEffectPreview && !_isDraggingForOrdering)
        {
          _registry.CardEffectPreviewService.DisplayPlayEffectPreview(playEffectPreview);
        }
        _draggedToClearThreshold = true;
      }
    }

    public override void MouseUp(bool isSameObject)
    {
      _registry.SoundService.PlayCardSound();
      _registry.CardService.ClearInfoZoom();
      _registry.CardEffectPreviewService.ClearPlayEffectPreview();
      _registry.CardService.IsPointerDownOnCard = false;

      if (CardView.Revealed?.Actions?.OnClick is { } onClick && isSameObject && (Time.time - _lastMouseDownTime < 1f))
      {
        _registry.ActionService.PerformAction(new GameAction
        {
          DebugAction = onClick.DebugAction,
          BattleAction = onClick.BattleAction,
        });
      }

      if (_isDraggingForOrdering)
      {
        _isDraggingFromHand = false;
        _isDraggingForOrdering = false;
        _registry.SoundService.PlayCardSound();
        var action = new GameAction
        {
          BattleAction = new()
          {
            BattleActionClass = new()
            {
              SelectCardOrder =
                  _registry.Layout.CardOrderSelector.SelectCardOrderWithinDisplay(transform, CardView.Id),
            }
          }
        };

        _registry.ActionService.PerformAction(action);
      }
      else if (ShouldReturnToPreviousParentOnRelease())
      {
        _registry.LayoutService.AddToParent(this);
        _registry.LayoutService.RunAnimations(() =>
        {
          _isDraggingFromHand = false;
        });
      }
      else if (_isDraggingFromHand)
      {
        _isDraggingFromHand = false;
        if (CardView.Revealed?.Actions?.OnPlaySound is { } onPlaySound)
        {
          _registry.SoundService.Play(onPlaySound);
        }
        else
        {
          _registry.SoundService.PlayWhooshSound();
        }
        var action = new GameAction
        {
          BattleAction = new()
          {
            BattleActionClass = new()
            {
              PlayCardFromHand = CardView.Id
            }
          }
        };

        _registry.ActionService.PerformAction(action);
      }
    }

    public override void MouseHoverStart()
    {
      if (_registry.CapabilitiesService.CanInfoZoom(GameContext) && GameContext != GameContext.Hand)
      {
        _hoverStartTime = Time.time;
        _hoveringForInfoZoom = true;
      }
    }

    public override void MouseHover()
    {
      if (Time.time - _hoverStartTime > 0.3f && _hoveringForInfoZoom && !_longHoverFired)
      {
        _registry.CardService.DisplayInfoZoom(this);
        _longHoverFired = true;
      }
    }

    public override void MouseHoverEnd()
    {
      if (_hoveringForInfoZoom)
      {
        _registry.CardService.ClearInfoZoom();
        _hoveringForInfoZoom = false;
        _longHoverFired = false;
      }
    }

    void ToggleActiveElements()
    {
      if (!_isRevealed)
      {
        _cardBack.gameObject.SetActive(true);
        _cardFront.gameObject.SetActive(false);
        _battlefieldCardFront.gameObject.SetActive(false);
      }
      else if (_isDissolved)
      {
        _cardBack.gameObject.SetActive(false);
        _cardFront.gameObject.SetActive(!GameContext.IsBattlefieldContext());
        _battlefieldCardFront.gameObject.SetActive(GameContext.IsBattlefieldContext());
        _cardFrame.gameObject.SetActive(!GameContext.IsBattlefieldContext());
        _battlefieldSparkBackground.gameObject.SetActive(false);
        _battlefieldOutline.gameObject.SetActive(false);
      }
      else if (HasGameContext && GameContext.IsBattlefieldContext())
      {
        _cardBack.gameObject.SetActive(false);
        _cardFront.gameObject.SetActive(false);
        _battlefieldOutline.gameObject.SetActive(GameContext != GameContext.DiscardPile);
        _battlefieldCardFront.gameObject.SetActive(true);
        _battlefieldSparkBackground.gameObject.SetActive(
            GameContext != GameContext.DiscardPile && CardView.Revealed?.Spark != null);
        _cardCollider.center = Vector3.zero;
        _cardCollider.size = new Vector3(2.5f, 3f, 0.1f);
      }
      else
      {
        _cardBack.gameObject.SetActive(false);
        _cardFront.gameObject.SetActive(true);
        _battlefieldCardFront.gameObject.SetActive(false);
        _sparkBackground.gameObject.SetActive(CardView.Revealed?.Spark != null);
        _cardCollider.center = new Vector3(0, -0.5f, 0);
        _cardCollider.size = new Vector3(2.5f, 4f, 0.1f);
      }
    }

    Vector3 HandCardJumpPosition()
    {
      if (_registry.IsLandscape)
      {
        // Bias slightly towards screen center
        var horizontalPosition = Mathf.Clamp01((transform.position.x - 8f) / 14f);
        return transform.position + Vector3.Lerp(
          new Vector3(2f, 6f, 2.5f),
          new Vector3(-2f, 6f, 2.5f),
          horizontalPosition);
      }
      else
      {
        // Keep card above user's finger on mobile so they can read it.
        var screenZ = Camera.main.WorldToScreenPoint(gameObject.transform.position).z;
        var worldPosition = _registry.InputService.WorldPointerPosition(screenZ);
        var offset = gameObject.transform.position - worldPosition;
        var target = transform.position + new Vector3(0, 3, Mathf.Max(1.75f, 3.25f - offset.z));
        target.x = Mathf.Clamp(target.x,
            _registry.Layout.InfoZoomLeft.position.x,
            _registry.Layout.InfoZoomRight.position.x);
        target.y = Mathf.Clamp(target.y, 20f, 25f);
        target.z = Mathf.Clamp(target.z, -25f, -20f);
        return target;
      }
    }

    bool ShouldReturnToPreviousParentOnRelease()
    {
      if (CardView.Revealed?.Actions.CanPlay != true && CardView.Revealed?.Actions.CanSelectOrder != true)
      {
        return true;
      }

      var mousePosition = _registry.InputService.WorldPointerPosition(_dragStartScreenZ);
      var zDistance = mousePosition.z - _dragStartPosition.z;
      return zDistance < 1f;
    }

    bool CanPlay() => CardView.Revealed?.Actions.CanPlay == true &&
      _registry.CapabilitiesService.CanPlayCards() &&
      GameContext == GameContext.Hand;

    bool CanSelectOrder() => CardView.Revealed?.Actions.CanSelectOrder == true && GameContext == GameContext.Browser;
  }
}