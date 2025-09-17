#nullable enable

using System;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine.UIElements;

namespace Dreamtides.Masonry
{
  public static class TextFields
  {
    public static void Apply(Registry _, NodeTextField field, TextFieldNode data)
    {
      field.SetGlobalIdentifierAndInitialText(data.GlobalIdentifier, data.InitialText);
      field.multiline = data.Multiline ?? false;
      field.isReadOnly = data.IsReadOnly ?? false;
      field.maxLength = data.MaxLength > 0 ? (int)data.MaxLength : -1;
      field.isPasswordField = data.IsPasswordField ?? false;
      field.doubleClickSelectsWord = data.DoubleClickSelectsWord ?? false;
      field.tripleClickSelectsLine = data.TripleClickSelectsLine ?? false;
      field.maskChar = data.MaskCharacter?.Length > 0 ? data.MaskCharacter[0] : '*';
    }
  }

  public sealed class NodeTextField : TextField, INodeCallbacks
  {
    public VisualElement Self => this;
    readonly Lazy<Callbacks> _callbacks = new();
    public Lazy<Callbacks> Callbacks => _callbacks;
    public FlexNode? Node { get; set; }
    string _globalIdentifier = "";

    public void SetGlobalIdentifierAndInitialText(string globalIdentifier, string initialText)
    {
      Errors.CheckArgument(globalIdentifier != "", "Global identifier cannot be empty");
      if (globalIdentifier != _globalIdentifier)
      {
        value = initialText;
        _globalIdentifier = globalIdentifier;
      }
    }
  }
}
