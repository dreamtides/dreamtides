#if UNITY_EDITOR

#nullable enable

using System;
using System.IO;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Editors
{
  /// <summary>
  /// Prefixes the Unity Editor window title with the worktree name (e.g.
  /// [ALPHA]) so that worktree editors are visually distinguishable. Has no
  /// effect in the main repository.
  /// </summary>
  [InitializeOnLoad]
  internal static class WorktreeWindowTitle
  {
    private static readonly string? WorktreeName;

    static WorktreeWindowTitle()
    {
      WorktreeName = ResolveWorktreeName();
      if (WorktreeName != null)
      {
        EditorApplication.updateMainWindowTitle += OnUpdateTitle;
        EditorApplication.UpdateMainWindowTitle();
      }
    }

    private static string? ResolveWorktreeName()
    {
      try
      {
        var repoRoot = Path.GetFullPath(Path.Combine(Application.dataPath, "..", ".."));
        var home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        var worktreeBase = Path.Combine(home, "dreamtides-worktrees");

        if (!repoRoot.StartsWith(worktreeBase + Path.DirectorySeparatorChar) &&
            !repoRoot.StartsWith(worktreeBase + "/"))
        {
          return null;
        }

        var relative = repoRoot.Substring(worktreeBase.Length + 1);
        var sep = relative.IndexOfAny(new[] { '/', Path.DirectorySeparatorChar });
        return sep >= 0 ? relative.Substring(0, sep) : relative;
      }
      catch (Exception e)
      {
        Debug.LogWarning($"[WorktreeWindowTitle] Failed to resolve worktree name: {e.Message}");
        return null;
      }
    }

    private static void OnUpdateTitle(ApplicationTitleDescriptor desc)
    {
      if (WorktreeName != null)
      {
        desc.title = $"[{WorktreeName.ToUpperInvariant()}] {desc.title}";
      }
    }
  }
}
#endif
