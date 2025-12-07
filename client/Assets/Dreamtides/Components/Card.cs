#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using DG.Tweening;
using Dreamtides.Buttons;
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
    [SerializeField]
    internal Transform _cardFront = null!;

    [SerializeField]
    internal Transform _battlefieldCardFront = null!;

    [SerializeField]
    internal CanvasCard _canvasCard = null!;

    [SerializeField]
    internal TextMeshPro _name = null!;

    [SerializeField]
    internal TextMeshPro _rulesText = null!;

    [SerializeField]
    internal TextMeshPro _typeText = null!;

    [SerializeField]
    internal Renderer _cardFrame = null!;

    [SerializeField]
    internal Renderer _cardFrameBackground = null!;

    [SerializeField]
    internal Renderer _cardImage = null!;

    [SerializeField]
    internal GameObject _shadowCaster = null!;

    [SerializeField]
    internal SpriteRenderer _battlefieldCardImage = null!;

    [SerializeField]
    internal BoxCollider _cardCollider = null!;

    [SerializeField]
    internal Renderer _cardBack = null!;

    [SerializeField]
    internal Renderer _outline = null!;

    [SerializeField]
    internal SpriteRenderer _battlefieldOutline = null!;

    [SerializeField]
    internal SpriteRenderer? _costBackground;

    [SerializeField]
    internal TextMeshPro? _costText;

    [SerializeField]
    internal TextMeshPro _producedEnergyText = null!;

    [SerializeField]
    internal Renderer? _sparkBackground;

    [SerializeField]
    internal TextMeshPro? _sparkText;

    [SerializeField]
    internal Renderer _battlefieldSparkBackground = null!;

    [SerializeField]
    internal TextMeshPro _battlefieldSparkText = null!;

    [SerializeField]
    internal ObjectLayout? _containedObjects;

    [SerializeField]
    internal ObjectLayout? _stackedObjects;

    [SerializeField]
    internal Transform _cardTrailPosition = null!;

    [SerializeField]
    internal GameObject _battlefieldIconContainer = null!;

    [SerializeField]
    internal TextMeshPro _battlefieldIcon = null!;

    [SerializeField]
    internal InfoZoomIcons _spriteCardInfoZoomIcons = null!;

    [SerializeField]
    internal InfoZoomIcons _battlefieldCardInfoZoomIcons = null!;

    [SerializeField]
    internal GameObject? _loopingEffect;

    [SerializeField]
    internal GameObject? _cardTrail;

    [SerializeField]
    internal DisplayableButton _buttonAttachment = null!;

    [SerializeField]
    internal SpriteRenderer? _spriteCardContentProtection;

    [SerializeField]
    internal float _cardColliderHeight = 4f;

    bool _isRevealed = false;
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

    EffectAddress? _loopingEffectAddress;

    float _distanceDragged;
    float _hoverStartTime;
    bool _hoveringForInfoZoom;
    bool _longHoverFired;
    bool _reverseDissolveOnAppearPlayed;

    public CardView CardView => Errors.CheckNotNull(_cardView);

    public CardView? NullableCardView => _cardView;

    public string Id => CardView.ClientId();

    public ObjectLayout ContainedObjects => Errors.CheckNotNull(_containedObjects);

    public SpriteRenderer? SpriteCardContentProtection => _spriteCardContentProtection;

    public DisplayableButton ButtonAttachment => _buttonAttachment;

    protected override void OnInitialize()
    {
      _buttonAttachment.Initialize(this);

      if (_containedObjects)
      {
        _containedObjects.Initialize(this);
      }
    }

    public void Render(CardView view, Sequence? sequence = null)
    {
      var name = view.Revealed?.Name.Replace("\n", " ") ?? "Hidden Card";
      _cardView = view;
      gameObject.name = $"[{Id}] {name}";

      if (view.Revealed != null)
      {
        if (_isRevealed || view.Backless)
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
        if (_isRevealed && !view.Backless)
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
      Flip(
        _cardBack,
        _cardFront,
        sequence,
        () =>
        {
          _isRevealed = false;
          _cardBack.gameObject.SetActive(true);
          _cardFront.gameObject.SetActive(false);
        }
      );
    }

    public void TurnFaceUp(Sequence? sequence = null)
    {
      Flip(
        _cardFront,
        _cardBack,
        sequence,
        () =>
        {
          _cardBack.gameObject.SetActive(false);
          _cardFront.gameObject.SetActive(true);
        }
      );
    }

    /// <summary>
    /// Hides the button attachment until the next Render() call.
    /// </summary>
    public void HideButtonAttachmentUntilNextRender()
    {
      _buttonAttachment.gameObject.SetActive(false);
    }

    public IEnumerator StartDissolve(DissolveCardCommand command)
    {
      _isDissolved = true;
      if (_sparkBackground)
      {
        _sparkBackground.enabled = false;
      }

      ToggleActiveElements();

      var dissolveSpeed = (float)(command.DissolveSpeed ?? 1f);
      var fadeDuration = 1f / dissolveSpeed;

      var textMeshProComponents = new List<TextMeshPro>();
      foreach (var textMeshPro in GetComponentsInChildren<TextMeshPro>())
      {
        if (textMeshPro.gameObject.activeInHierarchy)
        {
          textMeshProComponents.Add(textMeshPro);
          var color = textMeshPro.color;
          color.a = 0f;
          textMeshPro.color = color;
        }
      }

      var coroutines = new List<Coroutine>();
      foreach (var renderer in GetComponentsInChildren<Renderer>())
      {
        if (
          renderer.gameObject.activeInHierarchy
          && renderer.enabled
          && !renderer.GetComponent<TextMeshPro>()
        )
        {
          var effect = renderer.GetComponent<DissolveEffect>();
          if (!effect)
          {
            effect = renderer.gameObject.AddComponent<DissolveEffect>();
            effect.Initialize();
          }
          coroutines.Add(StartCoroutine(effect.StartDissolve(Registry, command)));
        }
      }

      yield return new WaitForSeconds(0.3f);

      foreach (var textMeshPro in textMeshProComponents)
      {
        if (textMeshPro != null)
        {
          textMeshPro.DOFade(1f, fadeDuration);
        }
      }

      foreach (var coroutine in coroutines)
      {
        yield return coroutine;
      }

      if (command.Reverse)
      {
        _isDissolved = false;
        ToggleActiveElements();
      }
    }

    public void ApplyPreview(CardPreviewView preview, Color textColor)
    {
      if (preview.Cost != null && _costText)
      {
        _costText.text = preview.Cost;
        _costText.color = textColor;
      }

      if (preview.Spark != null && _sparkText)
      {
        _sparkText.text = preview.Spark;
        _battlefieldSparkText.text = preview.Spark;
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
      if (_costText)
      {
        _costText.text = CardView.Revealed?.Cost;
        _costText.color = Color.white;
      }

      if (_sparkText)
      {
        _sparkText.text = CardView.Revealed?.Spark;
        _sparkText.color = Color.white;
      }

      _battlefieldSparkText.text = CardView.Revealed?.Spark;
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

      if (result._loopingEffect)
      {
        Destroy(result._loopingEffect);
        result._loopingEffect = null;
      }

      if (result._cardTrail)
      {
        Destroy(result._cardTrail);
        result._cardTrail = null;
      }

      result.gameObject.name = "[IZ]" + gameObject.name;
      result._cardView = CardView;
      result._outline.enabled = false;
      result._isRevealed = _isRevealed;
      result.Parent = null;
      result.GameContext = GameContext.InfoZoom;
      return result;
    }

    /// <summary>
    /// Sets the info zoom icon for this card, or clears the icon if the
    /// parameter is null.
    /// </summary>
    public void SetInfoZoomIcon(InfoZoomIcon? icon)
    {
      if (icon == null)
      {
        _battlefieldCardInfoZoomIcons.gameObject.SetActive(false);
        _spriteCardInfoZoomIcons.gameObject.SetActive(false);
      }
      else if (GameContext.IsBattlefieldContext())
      {
        _battlefieldCardInfoZoomIcons.SetText(icon.Icon, MasonRenderer.ToUnityColor(icon.Color));
        _battlefieldCardInfoZoomIcons.gameObject.SetActive(true);
        _spriteCardInfoZoomIcons.gameObject.SetActive(false);
      }
      else
      {
        _spriteCardInfoZoomIcons.SetText(icon.Icon, MasonRenderer.ToUnityColor(icon.Color));
        _spriteCardInfoZoomIcons.gameObject.SetActive(true);
        _battlefieldCardInfoZoomIcons.gameObject.SetActive(false);
      }
    }

    protected override void OnUpdate()
    {
      _outline.gameObject.SetActive(
        CanPlay() || CanSelectOrder() || CardView.Revealed?.OutlineColor != null
      );
    }

    void Flip(Component faceUp, Component faceDown, Sequence? sequence, Action? onFlipped = null)
    {
      if (sequence != null)
      {
        const float duration = TweenUtils.FlipAnimationDurationSeconds / 2f;
        sequence
          .Insert(0, faceDown.transform.DOLocalRotate(new Vector3(0, 90, 0), duration))
          .InsertCallback(
            duration,
            () =>
            {
              faceUp.transform.localRotation = Quaternion.Euler(0, -90, 0);
              onFlipped?.Invoke();
            }
          )
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

    public void SetCardTrail(ProjectileAddress trailAddress, float? durationSeconds = null)
    {
      if (_cardTrail)
      {
        Destroy(_cardTrail);
      }

      var trail = ComponentUtils.Instantiate(
        Registry.AssetService.GetProjectilePrefab(trailAddress)
      );
      _cardTrail = trail.gameObject;
      _cardTrail.transform.SetParent(_cardTrailPosition, worldPositionStays: false);
      _cardTrail.transform.localPosition = Vector3.zero;
      _cardTrail.transform.localRotation = Quaternion.identity;

      var trailComponent = _cardTrail.AddComponent<CardTrail>();
      trailComponent.Initialize(durationSeconds);
    }

    public void ClearCardTrail()
    {
      if (_cardTrail)
      {
        Destroy(_cardTrail);
        _cardTrail = null;
      }
    }

    void RenderRevealedCardView(RevealedCardView revealed)
    {
      _isRevealed = true;
      ToggleActiveElements();
      _canvasCard.RenderRevealedCardView(revealed);

      _name.text = revealed.Name;
      _rulesText.text = revealed.RulesText;
      if (_costBackground)
      {
        _costBackground.gameObject.SetActive(revealed.Cost != null);
      }
      if (_costText)
      {
        _costText.text = revealed.Cost?.ToString();
      }
      _producedEnergyText.text = revealed.Produced?.ToString();

      if (_sparkText)
      {
        _sparkText.text = revealed.Spark?.ToString();
      }

      _battlefieldSparkText.text = revealed.Spark?.ToString();
      _typeText.text = revealed.CardType;

      if (_cardImage is SpriteRenderer spriteRenderer && revealed.Image.Sprite != null)
      {
        spriteRenderer.sprite = Registry.AssetService.GetSprite(revealed.Image.Sprite);
      }
      else if (_cardImage is MeshRenderer meshRenderer && revealed.Image.Prefab != null)
      {
        var prefab = Registry.AssetService.GetPrefab(revealed.Image.Prefab.Prefab);
        Registry.StudioService.EndCapture(revealed.Image.Prefab.StudioType);
        Registry.StudioService.CaptureSubject(
          revealed.Image.Prefab.StudioType,
          prefab,
          meshRenderer,
          far: true
        );
      }
      else
      {
        Registry.LoggingService.LogError($"Card has no valid image", ("id", Id));
      }

      if (revealed.Image.Sprite is { } sprite)
      {
        _battlefieldCardImage.sprite = Registry.AssetService.GetSprite(sprite);
      }

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

      if (revealed.Effects?.LoopingEffect != null)
      {
        if (_loopingEffectAddress != revealed.Effects.LoopingEffect)
        {
          // Update the looping effect if it has changed or not already started.
          Destroy(_loopingEffect);
          _loopingEffectAddress = revealed.Effects.LoopingEffect;
          _loopingEffect = ComponentUtils
            .Instantiate(Registry.AssetService.GetEffectPrefab(_loopingEffectAddress), Vector3.zero)
            .gameObject;
          _loopingEffect.transform.SetParent(transform, worldPositionStays: false);
          // TODO: Figure out correct rotation for looping effect.
          _loopingEffect.transform.localEulerAngles = new Vector3(90, 0, 0);
        }
      }
      else
      {
        Destroy(_loopingEffect);
        _loopingEffect = null;
        _loopingEffectAddress = null;
      }

      if (revealed.Actions?.ButtonAttachment != null && HasGameContext)
      {
        _buttonAttachment.gameObject.SetActive(true);
        _buttonAttachment.GameContext = GameContext;
        _buttonAttachment.SetView(revealed.Actions.ButtonAttachment);
      }
      else
      {
        _buttonAttachment.gameObject.SetActive(false);
      }

      if (revealed.Effects?.ReverseDissolveOnAppear != null && !_reverseDissolveOnAppearPlayed)
      {
        _reverseDissolveOnAppearPlayed = true;
        StartCoroutine(StartDissolve(revealed.Effects.ReverseDissolveOnAppear));
      }
    }

    void RenderHiddenCardView()
    {
      _isRevealed = false;
      ToggleActiveElements();
    }

    public override bool CanHandleMouseEvents() =>
      GameContext != GameContext.Deck
      && GameContext != GameContext.DiscardPile
      && GameContext != GameContext.InfoZoom;

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
      Registry.CardAnimationService.IsPointerDownOnCard = true;

      if (
        Registry.IsMobileDevice
        && GameContext == GameContext.Hand
        && CardView.Position.Position.PositionClass?.InHand == DisplayPlayer.User
        && CardView.Revealed?.Actions?.OnClick?.ToGameAction() == null
        && !Registry.CapabilitiesService.AnyBrowserOpen()
      )
      {
        // Jump to large size when in user hand on mobile
        transform.position = MobileHandCardJumpPosition();
        transform.rotation = Quaternion.Euler(Constants.CameraXAngle, 0, 0);
        Registry.CardAnimationService.DisplayInfoZoom(this, forCardInHand: true);
      }
      else if (
        !Registry.IsMobileDevice
        && GameContext == GameContext.Hand
        && CardView.Position.Position.PositionClass?.InHand == DisplayPlayer.User
        && !Registry.CapabilitiesService.AnyBrowserOpen()
      )
      {
        var jumpPosition = Registry.UserHandHoverService.CalculateJumpPosition(this);
        if (jumpPosition != null)
        {
          transform.position = jumpPosition.Value;
          transform.rotation = Quaternion.Euler(Constants.CameraXAngle, 0, 0);
        }
        Registry.CardAnimationService.DisplayInfoZoom(this, forCardInHand: true);
      }
      else if (
        Registry.CapabilitiesService.CanInfoZoom(GameContext, CardView.Position.Position)
        && CardView.Revealed != null
        && !_draggedToClearThreshold
      )
      {
        Registry.CardAnimationService.DisplayInfoZoom(this, forCardInHand: false);
      }

      if (CanPlay() || CanSelectOrder())
      {
        _isDraggingFromHand =
          GameContext == GameContext.Hand || GameContext == GameContext.Hovering;
        _isDraggingForOrdering = CanSelectOrder();
        Registry.SoundService.PlayCardSound();
        GameContext = GameContext.Dragging;

        if (Parent)
        {
          Parent.RemoveIfPresent(this);
        }

        _dragStartScreenZ = Camera.main.WorldToScreenPoint(gameObject.transform.position).z;
        _dragStartPosition = Registry.InputService.WorldPointerPosition(_dragStartScreenZ);
        _dragOffset = gameObject.transform.position - _dragStartPosition;
      }
      else if (GameContext == GameContext.Hand && !Registry.CapabilitiesService.AnyBrowserOpen())
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

      var mousePositionInStartingPlane = Registry.InputService.WorldPointerPosition(
        _dragStartScreenZ
      );
      _distanceDragged = Vector2.Distance(mousePositionInStartingPlane, _dragStartPosition);

      if (_isDraggingForOrdering || Registry.IsLandscape)
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
          transform.position = Registry.InputService.WorldPointerPosition(20f);
        }
        else
        {
          float t = Mathf.Clamp01(_distanceDragged / playThreshold);
          Vector3 startPosition = _dragOffset + mousePositionInStartingPlane;
          Vector3 endPosition = Registry.InputService.WorldPointerPosition(20f);
          transform.position = Vector3.Lerp(startPosition, endPosition, t);
        }
      }

      if (_distanceDragged > 0.25f)
      {
        Registry.CardAnimationService.ClearInfoZoom();
        _draggedToClearThreshold = true;
      }

      if (_distanceDragged > 1f)
      {
        if (
          CardView.Revealed?.Actions.PlayEffectPreview is { } playEffectPreview
          && !_isDraggingForOrdering
        )
        {
          Registry.CardEffectPreviewService.DisplayBattlePreview(playEffectPreview);
        }
      }
    }

    public override void MouseUp(bool isSameObject)
    {
      Registry.SoundService.PlayCardSound();
      Registry.CardAnimationService.ClearInfoZoom();
      Registry.CardAnimationService.IsPointerDownOnCard = false;

      if (
        CardView.Revealed?.Actions?.OnClick is { } onClick
        && isSameObject
        && (Time.time - _lastMouseDownTime < 1f)
      )
      {
        Registry.ActionService.PerformAction(onClick.ToGameAction());
      }

      if (_isDraggingForOrdering)
      {
        _isDraggingFromHand = false;
        _isDraggingForOrdering = false;
        Registry.SoundService.PlayCardSound();
        var action = new GameAction
        {
          GameActionClass = new()
          {
            BattleAction = new()
            {
              BattleActionClass = new()
              {
                SelectOrderForDeckCard =
                  Registry.Layout.CardOrderSelector.SelectCardOrderWithinDisplay(
                    transform,
                    Errors.CheckNotNull(CardView.Revealed?.Actions?.CanSelectOrder)
                  ),
              },
            },
          },
        };

        Registry.ActionService.PerformAction(action);
      }
      else if (_isDraggingFromHand && ShouldReturnToPreviousParentOnRelease())
      {
        Registry.CardEffectPreviewService.ClearBattlePreview();
        Registry.CardService.AddToParent(this);
        Registry.CardService.RunAnimations(() =>
        {
          _isDraggingFromHand = false;
        });
      }
      else if (_isDraggingFromHand)
      {
        _isDraggingFromHand = false;
        if (CardView.Revealed?.Actions?.OnPlaySound is { } onPlaySound)
        {
          Registry.SoundService.Play(onPlaySound);
        }
        else
        {
          Registry.SoundService.PlayWhooshSound();
        }
        Registry.ActionService.PerformAction(
          Errors.CheckNotNull(CardView.Revealed?.Actions?.CanPlay?.ToGameAction())
        );
      }
    }

    public override void MouseHoverStart()
    {
      if (
        Registry.CapabilitiesService.CanInfoZoom(GameContext, CardView.Position.Position)
        && CardView.Revealed != null
        && GameContext != GameContext.Hovering
      )
      {
        _hoverStartTime = Time.time;
        _hoveringForInfoZoom = true;
      }
    }

    public override void MouseHover()
    {
      if (Time.time - _hoverStartTime > 0.15f && _hoveringForInfoZoom && !_longHoverFired)
      {
        Registry.CardAnimationService.DisplayInfoZoom(this, forCardInHand: false);
        _longHoverFired = true;
      }
    }

    public override void MouseHoverEnd()
    {
      if (_hoveringForInfoZoom)
      {
        Registry.CardAnimationService.ClearInfoZoom();
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
        _cardFrame.gameObject.SetActive(false);
      }
      else if (_isDissolved)
      {
        _cardBack.gameObject.SetActive(false);
        _cardFront.gameObject.SetActive(!BattlefieldMode());
        _battlefieldCardFront.gameObject.SetActive(BattlefieldMode());
        _cardFrame.gameObject.SetActive(!BattlefieldMode());
        _battlefieldSparkBackground.gameObject.SetActive(false);
        _battlefieldOutline.gameObject.SetActive(false);
      }
      else if (BattlefieldMode())
      {
        _cardBack.gameObject.SetActive(false);
        _cardFront.gameObject.SetActive(false);
        _battlefieldOutline.gameObject.SetActive(
          !CardView.Backless && GameContext != GameContext.DiscardPile
        );
        _battlefieldCardFront.gameObject.SetActive(true);
        _battlefieldSparkBackground.gameObject.SetActive(
          GameContext != GameContext.DiscardPile && CardView.Revealed?.Spark != null
        );
        _cardCollider.center = Vector3.zero;
        _cardCollider.size = new Vector3(2.5f, 3f, 0.1f);
        _cardFrame.gameObject.SetActive(false);
      }
      else
      {
        _cardBack.gameObject.SetActive(false);
        _cardFront.gameObject.SetActive(true);
        _battlefieldCardFront.gameObject.SetActive(false);

        if (_sparkBackground)
        {
          _sparkBackground.gameObject.SetActive(CardView.Revealed?.Spark != null);
        }
        _cardCollider.center = new Vector3(0, -0.5f, 0);
        _cardCollider.size = new Vector3(2.5f, _cardColliderHeight, 0.1f);
        _cardFrame.gameObject.SetActive(true);
      }

      UpdateShadowCasting();
    }

    void UpdateShadowCasting()
    {
      var disableShadows =
        CardView.Position.Position.Enum == PositionEnum.QuestDeck
        || CardView.Position.Position.Enum == PositionEnum.DreamsignDisplay;
      _shadowCaster.SetActive(!disableShadows);
      _cardBack.shadowCastingMode = disableShadows
        ? UnityEngine.Rendering.ShadowCastingMode.Off
        : UnityEngine.Rendering.ShadowCastingMode.On;
    }

    bool BattlefieldMode() => HasGameContext && GameContext.IsBattlefieldContext();

    Vector3 MobileHandCardJumpPosition()
    {
      // Keep card above user's finger on mobile so they can read it.
      var screenZ = Camera.main.WorldToScreenPoint(gameObject.transform.position).z;
      var worldPosition = Registry.InputService.WorldPointerPosition(screenZ);
      var offset = gameObject.transform.position - worldPosition;
      var target = transform.position + new Vector3(0, 3, Mathf.Max(1.75f, 3.25f - offset.z));
      target.x = Mathf.Clamp(
        target.x,
        Registry.Layout.InfoZoomLeft.position.x,
        Registry.Layout.InfoZoomRight.position.x
      );
      target.y = Mathf.Clamp(target.y, 20f, 25f);
      target.z = Mathf.Clamp(target.z, -25f, -20f);
      return target;
    }

    bool ShouldReturnToPreviousParentOnRelease()
    {
      if (
        CardView.Revealed?.Actions.CanPlay.HasValue != true
        && CardView.Revealed?.Actions.CanSelectOrder.HasValue != true
      )
      {
        return true;
      }

      var mousePosition = Registry.InputService.WorldPointerPosition(_dragStartScreenZ);
      var zDistance = mousePosition.z - _dragStartPosition.z;
      return zDistance < 1f;
    }

    bool CanPlay() =>
      CardView.Revealed?.Actions.CanPlay is { } canPlay
      && !canPlay.IsNull
      && Registry.CapabilitiesService.CanPlayCards()
      && (GameContext == GameContext.Hand || GameContext == GameContext.Hovering);

    bool CanSelectOrder() =>
      CardView.Revealed?.Actions.CanSelectOrder.HasValue == true
      && GameContext == GameContext.Browser;
  }
}
