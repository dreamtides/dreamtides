#nullable enable

using System;
using System.Collections;
using DG.Tweening;
using Dreamtides.Layout;
using Dreamtides.Masonry;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using TMPro;
using UnityEngine;

namespace Dreamtides.Components
{
  public class Card : Displayable
  {
    [SerializeField] Transform _cardFront = null!;
    [SerializeField] Transform _battlefieldCardFront = null!;
    [SerializeField] TextMeshPro _name = null!;
    [SerializeField] TextMeshPro _rulesText = null!;
    [SerializeField] TextMeshPro _typeText = null!;
    [SerializeField] Renderer _cardFrame = null!;
    [SerializeField] SpriteRenderer _cardImage = null!;
    [SerializeField] SpriteRenderer _battlefieldCardImage = null!;
    [SerializeField] DissolveEffect _cardImageDissolve = null!;
    [SerializeField] BoxCollider _cardCollider = null!;
    [SerializeField] Renderer _cardBack = null!;
    [SerializeField] Renderer _outline = null!;
    [SerializeField] SpriteRenderer _battlefieldOutline = null!;
    [SerializeField] Renderer _costBackground = null!;
    [SerializeField] TextMeshPro _costText = null!;
    [SerializeField] TextMeshPro _producedEnergyText = null!;
    [SerializeField] Renderer _sparkBackground = null!;
    [SerializeField] TextMeshPro _sparkText = null!;
    [SerializeField] Renderer _battlefieldSparkBackground = null!;
    [SerializeField] TextMeshPro _battlefieldSparkText = null!;
    [SerializeField] ObjectLayout? _containedObjects;
    [SerializeField] ObjectLayout? _stackedObjects;
    [SerializeField] Transform _cardTrailPosition = null!;
    [SerializeField] GameObject _battlefieldIconContainer = null!;
    [SerializeField] TextMeshPro _battlefieldIcon = null!;

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
      _outline.gameObject.SetActive(CanPlay() ||
          CanSelectOrder() ||
          (!GameContext.IsBattlefieldContext() && CardView.Revealed?.OutlineColor != null));
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
        GameContext != GameContext.Deck && GameContext != GameContext.DiscardPile;

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

      if (CardView.Revealed?.Actions?.OnClick is { } onClick && isSameObject && (Time.time - _lastMouseDownTime < 1f))
      {
        _registry.ActionService.PerformAction(new UserAction
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
        var action = new UserAction
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
      else if (_isDraggingFromHand && ShouldReturnToPreviousParentOnRelease())
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
        var action = new UserAction
        {
          BattleAction = new()
          {
            BattleActionClass = new()
            {
              PlayCard = CardView.Id
            }
          }
        };

        _registry.ActionService.PerformAction(action);
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
        _cardCollider.size = new Vector3(2.5f, 3f, 1f);
      }
      else
      {
        _cardBack.gameObject.SetActive(false);
        _cardFront.gameObject.SetActive(true);
        _battlefieldCardFront.gameObject.SetActive(false);
        _sparkBackground.gameObject.SetActive(CardView.Revealed?.Spark != null);
        _cardCollider.center = new Vector3(0, -0.5f, 0);
        _cardCollider.size = new Vector3(2.5f, 4f, 1f);
      }
    }

    Vector3 HandCardJumpPosition()
    {
      if (_registry.IsLandscape)
      {
        // Bias slightly towards screen center
        var horizontalPosition = Mathf.Clamp01((transform.position.x - 8f) / 14f);
        return transform.position + Vector3.Lerp(
          new Vector3(2f, 4f, 2f),
          new Vector3(-2f, 4f, 2f),
          horizontalPosition);
      }
      else
      {
        // Keep card above user's finger on mobile so they can read it.
        var screenZ = Camera.main.WorldToScreenPoint(gameObject.transform.position).z;
        var worldPosition = _registry.InputService.WorldPointerPosition(screenZ);
        var offset = gameObject.transform.position - worldPosition;
        var target = transform.position + new Vector3(0, 3, Mathf.Max(1.75f, 3.25f - offset.z));
        target.x = Mathf.Clamp(target.x, -1f, 1f);
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