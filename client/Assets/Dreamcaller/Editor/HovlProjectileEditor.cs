#nullable enable

using System.Linq;
using CGT.Pooling;
using Dreamcaller.Components;
using Dreamcaller.Layout;
using UnityEditor;
using UnityEngine;

namespace Dreamcaller.Editors
{
  [CustomEditor(typeof(HS_ProjectileMover))]
  public sealed class HovlProjectileEditor : Editor
  {
    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();

      if (GUILayout.Button("Upgrade"))
      {
        var projectile = (HS_ProjectileMover)target;
        var added = projectile.gameObject.AddComponent<Projectile>();

        // TODO: The Hovl projectiles now store their flash & hit effects as
        // children of the current component, which breaks some assumptions of
        // our system (e.g. the hit doesn't have "play on awake"). We need to
        // either standardize on this pattern or have the upgrade script delete
        // these children and directly reference prefabs.

        TimedEffect? flash = null;
        if (projectile.flash)
        {
          flash = projectile.flash.AddComponent<TimedEffect>();
        }

        TimedEffect? hit = null;
        if (projectile.hit)
        {
          hit = projectile.hit.AddComponent<TimedEffect>();
        }

        added._flash = flash;
        added._hit = hit;
        added._useFirePointRotation = projectile.UseFirePointRotation;
        added._rotationOffset = projectile.rotationOffset;
        added._hitParticleSystem = projectile.hitPS;
        added._light = projectile.lightSourse;
        added._detached = projectile.Detached.ToList();
        added._projectileParticleSystem = projectile.projectilePS;

        // Set Play on Awake for all particle systems
        UpdateParticleSystems(projectile.gameObject);
        if (hit != null)
        {
          UpdateParticleSystems(hit.gameObject);
        }
        if (flash != null)
        {
          UpdateParticleSystems(flash.gameObject);
        }

        CleanUpChildren(projectile.transform);
        DestroyImmediate(projectile.gameObject.GetComponent<Rigidbody>(), allowDestroyingAssets: true);
        DestroyImmediate(projectile.gameObject.GetComponent<SphereCollider>(), allowDestroyingAssets: true);
        DestroyImmediate(projectile.gameObject.GetComponent<HS_Poolable>(), allowDestroyingAssets: true);
        DestroyImmediate(projectile, allowDestroyingAssets: true);
      }
    }

    private void UpdateParticleSystems(GameObject gameObject)
    {
      var particleSystems = gameObject.GetComponentsInChildren<ParticleSystem>(includeInactive: true);
      foreach (var ps in particleSystems)
      {
        var main = ps.main;
        main.playOnAwake = true;
        main.scalingMode = ParticleSystemScalingMode.Hierarchy;
        var renderer = ps.GetComponent<Renderer>();
        if (renderer != null)
        {
          renderer.sortingLayerID = GameContext.Effects.SortingLayerId();
        }
      }
    }

    private void CleanUpChildren(Transform transform)
    {
      var callbackComponent = transform.GetComponent<HS_CallBackParent>();
      if (callbackComponent != null)
      {
        DestroyImmediate(callbackComponent, allowDestroyingAssets: true);
      }

      // Recursively process all children
      for (int i = 0; i < transform.childCount; i++)
      {
        CleanUpChildren(transform.GetChild(i));
      }
    }
  }
}