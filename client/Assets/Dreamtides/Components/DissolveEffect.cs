#nullable enable

using System.Collections;
using AmazingAssets.AdvancedDissolve;
using Dreamtides.Masonry;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Components
{
  public class DissolveEffect : MonoBehaviour
  {
    Renderer _target = null!;
    Material? _originalMaterial;
    bool _running = false;
    float _clipValue = 0;
    bool _reverse = false;
    float _speed = 1f;
    Registry? _registry;
    AudioClipAddress? _sound;
    bool _soundPlayed = false;

    public void Initialize()
    {
      _target = ComponentUtils.Get<Renderer>(gameObject);
      _originalMaterial = _target.material;
    }

    public IEnumerator StartDissolve(Registry registry, DissolveCardCommand command)
    {
      _reverse = command.Reverse;
      _speed = (float)(command.DissolveSpeed ?? 1f);
      _registry = registry;
      _sound = command.Sound;
      _soundPlayed = false;

      var material = Instantiate(registry.AssetService.GetMaterial(command.Material));
      material.mainTexture = _target.material.mainTexture;
      AdvancedDissolveKeywords.SetKeyword(material, AdvancedDissolveKeywords.State.Enabled, true);
      AdvancedDissolveProperties.Cutout.Standard.UpdateLocalProperty(
        material,
        AdvancedDissolveProperties.Cutout.Standard.Property.Clip,
        _reverse ? 1f : 0f
      );
      AdvancedDissolveProperties.Edge.Base.UpdateLocalProperty(
        material,
        AdvancedDissolveProperties.Edge.Base.Property.Color,
        MasonRenderer.ToUnityColor(command.Color)
      );
      AdvancedDissolveProperties.Edge.Base.UpdateLocalProperty(
        material,
        AdvancedDissolveProperties.Edge.Base.Property.ColorIntensity,
        Random.Range(5f, 8f)
      );
      AdvancedDissolveProperties.Edge.Base.UpdateLocalProperty(
        material,
        AdvancedDissolveProperties.Edge.Base.Property.Shape,
        AdvancedDissolveProperties.Edge.Base.Shape.Smoother
      );
      _target.material = material;
      _running = true;
      yield return new WaitUntil(() => !_running);
    }

    void Update()
    {
      if (!_running)
      {
        return;
      }

      _clipValue += Time.deltaTime * _speed * (_reverse ? -1 : 1);
      _clipValue = Mathf.Clamp01(_clipValue);
      AdvancedDissolveProperties.Cutout.Standard.UpdateLocalProperty(
        _target.material,
        AdvancedDissolveProperties.Cutout.Standard.Property.Clip,
        _clipValue
      );

      if (!_soundPlayed && _sound != null && _registry != null)
      {
        var halfwayReached = _reverse ? _clipValue <= 0.5f : _clipValue >= 0.5f;
        if (halfwayReached)
        {
          _registry.SoundService.Play(_sound);
          _soundPlayed = true;
        }
      }

      if (_clipValue >= 1 || _clipValue <= 0)
      {
        if (_reverse)
        {
          _target.material = _originalMaterial;
        }
        _running = false;
      }
    }
  }
}
