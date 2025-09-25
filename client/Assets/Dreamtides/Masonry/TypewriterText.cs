#nullable enable

using System;
using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine.UIElements;

namespace Dreamtides.Masonry
{
  public static class TypewriterText
  {
    public static void Apply(Registry registry, NodeTypewriterText view, TypewriterTextNode data)
    {
      var sound = data.SoundEffect;
      view.Configure(registry, data.Label, data.CharacterDelay.MillisecondsValue, sound);
    }
  }

  public sealed class NodeTypewriterText : Label, INodeCallbacks
  {
    public VisualElement Self => this;
    readonly Lazy<Callbacks> _callbacks = new();
    public Lazy<Callbacks> Callbacks => _callbacks;
    public FlexNode? Node { get; set; }

    Registry? _registry;
    string _fullText = string.Empty;
    int _currentIndex;
    bool _animationComplete;
    bool _started;
    bool _attachRegistered;
    long _characterDelayMilliseconds;
    IVisualElementScheduledItem? _scheduledItem;
    AudioClipAddress? _sound;

    public void Configure(
      Registry registry,
      string label,
      long characterDelayMilliseconds,
      AudioClipAddress? sound
    )
    {
      _registry = registry;

      if (_animationComplete)
      {
        text = _fullText;
        return;
      }

      _fullText = label ?? string.Empty;
      _characterDelayMilliseconds = characterDelayMilliseconds;
      _sound = sound; // Reserved for future sound effect integration.

      // Not yet started or completed – prepare for animation.
      if (!_started)
      {
        text = string.Empty;
      }

      // Register attach callback once so we start when the element is actually in a panel.
      if (!_attachRegistered)
      {
        _attachRegistered = true;
        RegisterCallback<AttachToPanelEvent>(_ => StartTypewriterIfNeeded());
      }

      // If we're already attached (Apply called after being added to panel), start immediately.
      if (panel != null)
      {
        StartTypewriterIfNeeded();
      }
    }

    void StartTypewriterIfNeeded()
    {
      if (_started || _animationComplete)
      {
        return;
      }
      _started = true;

      _currentIndex = 0;
      text = string.Empty;

      _scheduledItem = schedule
        .Execute(OnTick)
        .Every((long)Math.Max(1, _characterDelayMilliseconds));
    }

    void OnTick()
    {
      if (_animationComplete)
      {
        _scheduledItem?.Pause();
        return;
      }

      if (_currentIndex < _fullText.Length)
      {
        _currentIndex++;
        text = _fullText.Substring(0, _currentIndex);
        if (_registry != null && _sound != null)
        {
          _registry.SoundService.Play(_sound);
        }
      }

      if (_currentIndex >= _fullText.Length)
      {
        _animationComplete = true;
        _scheduledItem?.Pause();
      }
    }
  }
}
