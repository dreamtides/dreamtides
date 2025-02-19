#nullable enable

using System.Collections;
using AmazingAssets.AdvancedDissolve;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class DissolveEffect : MonoBehaviour
  {
    [SerializeField] MeshRenderer _meshRenderer;
    [SerializeField] Material _dissolveMaterial;
    [SerializeField] Color _edgeColor;
    [SerializeField] float _dissolveSpeed = 0.1f;
    bool _running = false;
    float _clipValue = 0;

    public IEnumerator StartDissolve()
    {
      var material = Instantiate(_dissolveMaterial);
      _meshRenderer.material = material;
      AdvancedDissolveKeywords.SetKeyword(
          material,
          AdvancedDissolveKeywords.State.Enabled, true);
      AdvancedDissolveProperties.Cutout.Standard.UpdateLocalProperty(
          material,
          AdvancedDissolveProperties.Cutout.Standard.Property.Clip, 0);
      AdvancedDissolveProperties.Edge.Base.UpdateLocalProperty(
          material,
          AdvancedDissolveProperties.Edge.Base.Property.Color, _edgeColor);
      AdvancedDissolveProperties.Edge.Base.UpdateLocalProperty(
          material,
          AdvancedDissolveProperties.Edge.Base.Property.ColorIntensity,
          Random.Range(5f, 8f));
      AdvancedDissolveProperties.Edge.Base.UpdateLocalProperty(
          material,
          AdvancedDissolveProperties.Edge.Base.Property.Shape,
          AdvancedDissolveProperties.Edge.Base.Shape.Smoother);

      _running = true;
      yield return new WaitUntil(() => !_running);
    }

    void Update()
    {
      if (!_running)
      {
        return;
      }

      AdvancedDissolveProperties.Cutout.Standard.UpdateLocalProperty(
        _meshRenderer.material,
        AdvancedDissolveProperties.Cutout.Standard.Property.Clip,
        _clipValue);
      _clipValue += Time.deltaTime * _dissolveSpeed;
      if (_clipValue >= 1)
      {
        _running = false;
      }
    }
  }
}