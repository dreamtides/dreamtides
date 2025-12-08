#nullable enable

using UnityEngine;

namespace Dreamtides.Utils
{
  public static class MaterialUtils
  {
    public static Material GetMaterial(Renderer renderer)
    {
      return Application.isPlaying ? renderer.material : renderer.sharedMaterial;
    }

    public static void SetMaterial(Renderer renderer, Material material)
    {
      if (Application.isPlaying)
      {
        renderer.material = material;
      }
      else
      {
        renderer.sharedMaterial = material;
      }
    }
  }
}
