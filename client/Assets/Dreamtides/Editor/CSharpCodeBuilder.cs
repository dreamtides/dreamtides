#nullable enable

using System;
using System.Collections.Generic;
using System.Text;
using UnityEngine;

namespace Dreamtides.Editors
{
  public class CSharpCodeBuilder
  {
    readonly StringBuilder _sb = new();
    int _indentLevel;
    bool _lineStarted;

    public CSharpCodeBuilder Indent()
    {
      _indentLevel++;
      return this;
    }

    public CSharpCodeBuilder Unindent()
    {
      _indentLevel = Math.Max(0, _indentLevel - 1);
      return this;
    }

    public CSharpCodeBuilder Line(string text = "")
    {
      EnsureIndent();
      _sb.AppendLine(text);
      _lineStarted = false;
      return this;
    }

    public CSharpCodeBuilder Append(string text)
    {
      EnsureIndent();
      _sb.Append(text);
      return this;
    }

    public CSharpCodeBuilder OpenBrace()
    {
      Line("{");
      Indent();
      return this;
    }

    public CSharpCodeBuilder CloseBrace(bool withSemicolon = false)
    {
      Unindent();
      Line(withSemicolon ? "};" : "}");
      return this;
    }

    public CSharpCodeBuilder BlankLine()
    {
      _sb.AppendLine();
      _lineStarted = false;
      return this;
    }

    public CSharpCodeBuilder Using(string ns)
    {
      Line($"using {ns};");
      return this;
    }

    public CSharpCodeBuilder Namespace(string ns)
    {
      Line($"namespace {ns}");
      return this;
    }

    public CSharpCodeBuilder Class(
      string name,
      string? baseClass = null,
      string visibility = "public"
    )
    {
      var inheritance = baseClass != null ? $" : {baseClass}" : "";
      Line($"{visibility} class {name}{inheritance}");
      return this;
    }

    public CSharpCodeBuilder Method(
      string returnType,
      string name,
      string parameters = "",
      string visibility = "public",
      bool isStatic = false
    )
    {
      var staticMod = isStatic ? "static " : "";
      Line($"{visibility} {staticMod}{returnType} {name}({parameters})");
      return this;
    }

    public CSharpCodeBuilder Var(string name, string value)
    {
      Line($"var {name} = {value};");
      return this;
    }

    public CSharpCodeBuilder Assign(string target, string value)
    {
      Line($"{target} = {value};");
      return this;
    }

    public CSharpCodeBuilder Return(string value)
    {
      Line($"return {value};");
      return this;
    }

    public CSharpCodeBuilder NewObject(string varName, string typeName, string args = "")
    {
      var argsStr = string.IsNullOrEmpty(args) ? "" : args;
      Line($"var {varName} = new {typeName}({argsStr});");
      return this;
    }

    public CSharpCodeBuilder AddComponent(string varName, string goVar, string typeName)
    {
      Line($"var {varName} = {goVar}.AddComponent<{typeName}>();");
      return this;
    }

    public CSharpCodeBuilder CreateGameObject(string varName, string name)
    {
      Line($"var {varName} = new GameObject(\"{name}\");");
      return this;
    }

    public CSharpCodeBuilder Call(string target, string method, params string[] args)
    {
      var argsStr = string.Join(", ", args);
      Line($"{target}.{method}({argsStr});");
      return this;
    }

    public static string ToLiteral(float value) => $"{value}f";

    public static string ToLiteral(int value) => value.ToString();

    public static string ToLiteral(bool value) => value ? "true" : "false";

    public static string ToLiteral(string value) => $"\"{EscapeString(value)}\"";

    public static string ToVector3(Vector3 v) =>
      $"new Vector3({ToLiteral(v.x)}, {ToLiteral(v.y)}, {ToLiteral(v.z)})";

    public static string ToQuaternion(Quaternion q) =>
      $"Quaternion.Euler({ToLiteral(q.eulerAngles.x)}, {ToLiteral(q.eulerAngles.y)}, {ToLiteral(q.eulerAngles.z)})";

    public static string ToColor(Color c) =>
      $"new Color({ToLiteral(c.r)}, {ToLiteral(c.g)}, {ToLiteral(c.b)}, {ToLiteral(c.a)})";

    public static string ToEnumValue(Type enumType, int value)
    {
      var name = Enum.GetName(enumType, value);
      return $"{enumType.Name}.{name}";
    }

    static string EscapeString(string s)
    {
      return s.Replace("\\", "\\\\")
        .Replace("\"", "\\\"")
        .Replace("\n", "\\n")
        .Replace("\r", "\\r");
    }

    void EnsureIndent()
    {
      if (!_lineStarted)
      {
        for (var i = 0; i < _indentLevel; i++)
        {
          _sb.Append("  ");
        }
        _lineStarted = true;
      }
    }

    public override string ToString() => _sb.ToString();
  }
}
