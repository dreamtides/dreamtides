#nullable enable

using System.Linq;
using CGT.Pooling;
using Dreamcaller.Components;
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

        CleanUpChildren(projectile.transform);
        DestroyImmediate(projectile.gameObject.GetComponent<Rigidbody>(), allowDestroyingAssets: true);
        DestroyImmediate(projectile.gameObject.GetComponent<SphereCollider>(), allowDestroyingAssets: true);
        DestroyImmediate(projectile.gameObject.GetComponent<HS_Poolable>(), allowDestroyingAssets: true);
        DestroyImmediate(projectile, allowDestroyingAssets: true);
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