#nullable enable

using System;
using System.Collections;
using System.Runtime.CompilerServices;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;
using UnityEngine.Rendering;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public abstract class Displayable : MonoBehaviour
  {
    [SerializeField]
    internal Transform? _projectileSourcePosition;

    [SerializeField]
    internal Transform? _displayEffectPosition;

    [SerializeField]
    internal SortingGroup? _sortingGroup;

    [SerializeField]
    internal GameContext _internalGameContext;

    ObjectLayout? _parent;
    ObjectPosition? _objectPosition;
    bool _initialized;
    Registry _registry = null!;
    GameMode _mode = GameMode.MainMenu;
    TestConfiguration? _testConfiguration;

    protected Registry Registry =>
      _initialized ? _registry : throw new InvalidOperationException($"{name} not initialized!");

    protected GameMode Mode =>
      _initialized ? _mode : throw new InvalidOperationException($"{name} not initialized!");

    protected TestConfiguration? TestConfiguration =>
      _initialized
        ? _testConfiguration
        : throw new InvalidOperationException($"{name} not initialized!");

    public void Initialize(
      Registry registry,
      GameMode mode,
      TestConfiguration? testConfiguration,
      bool fromRegistry = false
    )
    {
      if (_initialized)
      {
        throw new InvalidOperationException($"{name} already initialized!");
      }

      _initialized = true;
      _registry = registry;
      _mode = mode;
      _testConfiguration = testConfiguration;
      OnInitialize();

      if (!fromRegistry && gameObject.activeInHierarchy)
      {
        StartCoroutine(StartAsync());
      }
    }

    public void Initialize(Displayable displayable)
    {
      Initialize(displayable.Registry, displayable.Mode, displayable.TestConfiguration);
    }

    /// <summary>
    /// Invoked at start after all objects are initialized.
    /// </summary>
    public IEnumerator? StartFromRegistry()
    {
      if (!_initialized)
      {
        throw new InvalidOperationException($"{name} not initialized!");
      }

      OnStart();
      return OnStartAsync();
    }

    /// <summary>
    /// Do not override or hide this method. Use <see cref="OnInitialize"/>
    /// instead.
    /// </summary>
    public void Start() { }

    /// <summary>
    /// Invoked when the object is initialized.
    /// </summary>
    protected virtual void OnInitialize() { }

    IEnumerator StartAsync()
    {
      yield return new WaitForEndOfFrame();
      OnStart();
      var routine = OnStartAsync();
      if (routine != null)
      {
        yield return routine;
      }
    }

    /// <summary>
    /// Invoked at start after all objects are initialized.
    /// </summary>
    protected virtual void OnStart() { }

    /// <summary>
    /// Invoked as a couroutine at start after all objects are initialized.
    /// </summary>
    protected virtual IEnumerator? OnStartAsync() => null;

    /// <summary>
    /// If true, this object will not be modified by the layout system.
    /// </summary>
    public bool ExcludeFromLayout { get; set; }

    /// <summary>Order items within a sorting layer.</summary>
    public int SortingKey
    {
      get
      {
        if (_sortingGroup)
        {
          return _sortingGroup.sortingOrder;
        }
        else
        {
          return 0;
        }
      }
      set
      {
        if (_sortingGroup)
        {
          _sortingGroup.sortingOrder = value;
        }
      }
    }

    public ObjectLayout? Parent
    {
      get => _parent;
      set => _parent = value;
    }

    public ObjectPosition? ObjectPosition
    {
      get => _objectPosition;
      set => _objectPosition = value;
    }

    public virtual float DefaultScale => 1.0f;

    /// <summary>
    /// The position that a projectile will be fired from this object.
    /// </summary>
    public Transform? ProjectileSourcePosition => _projectileSourcePosition;

    /// <summary>
    /// Position at which to display particle effects on this object.
    /// </summary>
    public Transform? DisplayEffectPosition => _displayEffectPosition;

    /// <summary>
    /// SortingGroup for this object.
    /// </summary>
    public SortingGroup? SortingGroup => _sortingGroup;

    public void Update()
    {
      if (_initialized)
      {
        OnUpdate();
      }
    }

    /// <summary>
    /// Invoked every frame once the object is initialized.
    /// </summary>
    protected virtual void OnUpdate() { }

    /// <summary>
    /// Should return true if this game object can currently handle a MouseDown or MouseHoverStart event.
    /// </summary>
    public virtual bool CanHandleMouseEvents() => false;

    /// <summary>
    /// Invoked on mouse down. Will only be invoked if <see cref="CanHandleMouseEvents"/>
    /// returns true and this is the topmost object hit by the on click raycast.
    /// </summary>
    public virtual void MouseDown() { }

    /// <summary>
    /// Sent every frame while the mouse button is held down to objects which received <see cref="MouseDown"/>.
    /// </summary>
    public virtual void MouseDrag() { }

    /// <summary>
    /// Sent on *any* mouse up event, anywhere on screen, to objects which
    /// received a <see cref="MouseDown"/>. If "isSameObject" is true, it means
    /// the pointer was released over the same Displayable which received
    /// MouseDown.
    /// </summary>
    public virtual void MouseUp(bool isSameObject) { }

    /// <summary>
    /// Invoked on mouse hover start. Will only be invoked if <see cref="CanHandleMouseEvents"/>
    /// returns true and this is the topmost object hit by the on click raycast.
    /// </summary>
    public virtual void MouseHoverStart() { }

    public virtual void MouseHover() { }

    public virtual void MouseHoverEnd() { }

    /// <summary>Called when the parent container is repositioned.</summary>
    public virtual void OnParentRepositioned() { }

    public GameContext GameContext
    {
      get =>
        Errors.CheckNotDefault(
          _internalGameContext,
          $"internalGameContext not initialized for {name}"
        );
      set
      {
        Errors.CheckNotDefault(value);

        if (_internalGameContext != value)
        {
          var oldContext = _internalGameContext;
          _internalGameContext = value;
          if (_sortingGroup)
          {
            _sortingGroup.sortingLayerID = value.SortingLayerId();
          }
          OnSetGameContext(oldContext, value);
        }
      }
    }

    public bool HasGameContext => _internalGameContext != GameContext.Unspecified;

    public bool IsLandscape() =>
      _initialized ? Registry.GameViewport.IsLandscape : Screen.width > Screen.height;

    public bool IsMobileDevice() =>
      Application.isPlaying && _initialized
        ? Registry.IsMobileDevice
        : UnityEngine.Device.Application.isMobilePlatform;

    protected virtual void OnSetGameContext(GameContext oldContext, GameContext newContext) { }

#if UNITY_EDITOR
    void Reset()
    {
      var registries = FindObjectsByType<Registry>(
        FindObjectsInactive.Include,
        FindObjectsSortMode.None
      );
      var currentScene = gameObject.scene;
      var count = 0;
      var found = (Registry)null!;
      foreach (var r in registries)
      {
        if (r.gameObject.scene == currentScene)
        {
          count++;
          found = r;
          if (count > 1)
          {
            break;
          }
        }
      }
      if (count == 1)
      {
        _registry = found;
        return;
      }
      if (count == 0)
      {
        return;
      }
      throw new InvalidOperationException("Multiple Registry components found in this scene.");
    }
#endif
  }
}
