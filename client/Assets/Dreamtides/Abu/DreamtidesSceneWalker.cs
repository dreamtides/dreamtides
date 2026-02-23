#nullable enable

using System;
using System.Collections.Generic;
using System.Text;
using System.Text.RegularExpressions;
using Abu;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;

namespace Dreamtides.Abu
{
  /// <summary>
  /// Walks all Dreamtides UI systems and produces a structured accessibility
  /// snapshot tree. In battle mode, the tree is organized by zone (hand,
  /// battlefield, etc.) with semantic labels. Outside battle, falls back to
  /// a flat UIToolkit + 3D walk.
  /// </summary>
  public partial class DreamtidesSceneWalker : ISceneWalker
  {
    static readonly Regex RichTextTagPattern = new("<[^>]+>");

    readonly Registry _registry;

    public DreamtidesSceneWalker(Registry registry)
    {
      _registry = registry;
    }

    public AbuSceneNode Walk(RefRegistry refRegistry)
    {
      var root = CreateNode("application", "Dreamtides");

      if (_registry.BattleLayout.Contents.activeSelf)
      {
        root.Children.Add(WalkBattle(refRegistry));
      }
      else
      {
        root.Children.Add(WalkUiToolkit(refRegistry));
        root.Children.Add(WalkFallbackScene3D(refRegistry));
      }

      return root;
    }

    // ── Rich text stripping ───────────────────────────────────────────

    public static string StripRichText(string? text)
    {
      if (string.IsNullOrEmpty(text))
      {
        return "";
      }

      var stripped = RichTextTagPattern.Replace(text, "");
      var sb = new StringBuilder(stripped.Length);
      for (var i = 0; i < stripped.Length; i++)
      {
        var c = stripped[i];
        var code = (int)c;
        // Filter icon glyphs mapped to CJK code points by icon fonts
        if (code >= 0x3400 && code <= 0x9FFF)
        {
          continue;
        }

        // Filter icon glyphs (PUA + CJK compat / presentation forms / specials)
        if (code >= 0xE000 && code <= 0xFFFF)
        {
          continue;
        }

        // Filter Supplementary Private Use Area (represented as surrogate pairs)
        if (
          char.IsHighSurrogate(c)
          && i + 1 < stripped.Length
          && char.IsLowSurrogate(stripped[i + 1])
        )
        {
          var codePoint = char.ConvertToUtf32(c, stripped[i + 1]);
          if (codePoint >= 0xF0000 && codePoint <= 0xFFFFF)
          {
            i++; // skip low surrogate
            continue;
          }
        }

        sb.Append(c);
      }

      return sb.ToString().Trim();
    }

    static AbuSceneNode CreateNode(string role, string? label, bool interactive = false)
    {
      return new AbuSceneNode
      {
        Role = role,
        Label = label,
        Interactive = interactive,
      };
    }

    static AbuSceneNode CreateGroupNode(string label)
    {
      return CreateNode("group", label);
    }

    static AbuSceneNode CreateRegionNode(string label)
    {
      return CreateNode("region", label);
    }

    static AbuSceneNode CreateLabelNode(string label)
    {
      return CreateNode("label", label);
    }

    static AbuSceneNode CreateButtonNode(string label)
    {
      return CreateNode("button", label, interactive: true);
    }

    static string ToSingleLineText(string? value, string fallback = "")
    {
      var stripped = StripRichText(value);
      if (string.IsNullOrEmpty(stripped))
      {
        return fallback;
      }

      return stripped.Replace("\n", ", ");
    }

    void AddInteractiveNode(
      AbuSceneNode parent,
      RefRegistry refRegistry,
      string role,
      string label,
      RefCallbacks callbacks
    )
    {
      refRegistry.Register(callbacks);
      parent.Children.Add(CreateNode(role, label, interactive: true));
    }

    string AddInteractiveNodeWithRef(
      AbuSceneNode parent,
      RefRegistry refRegistry,
      string role,
      string label,
      RefCallbacks callbacks
    )
    {
      var reference = refRegistry.Register(callbacks);
      parent.Children.Add(CreateNode(role, label, interactive: true));
      return reference;
    }

    static HandCounts CountHandCards(IReadOnlyList<Displayable> handObjects)
    {
      var cardCount = 0;
      var abilityCount = 0;
      foreach (var obj in handObjects)
      {
        if (obj is Card card && card.CardView.Prefab == CardPrefab.Token)
        {
          abilityCount++;
        }
        else
        {
          cardCount++;
        }
      }

      return new HandCounts(cardCount, abilityCount);
    }

    readonly struct HandCounts
    {
      public HandCounts(int cardCount, int abilityCount)
      {
        CardCount = cardCount;
        AbilityCount = abilityCount;
      }

      public int CardCount { get; }

      public int AbilityCount { get; }
    }
  }
}
