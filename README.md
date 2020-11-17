# manokara's iced widgets

A collection of widgets for [iced].

## Building

Add the following to your `[dependencies]` section in `Cargo.toml`:

```
iced_widgets = { git = "https://github.com/manokara/iced_widgets" }
```

While there is not a tagged release yet, that crate spec will give you the latest commit and keep it
at that unless you run `cargo update`.

### Optional widgets

You can save compile time by only building the widgets you're going to use. You do that by removing
the default features and manually listing the widgets you want to use with features (see in the
Widget List below). Change the previous crate spec to this:

```
iced_widgets = { git = "https://github.com/manokara/iced_widgets", default-features = false, features = ["hexview"] }
```

To only include the Hexview widget. You can also turn the spec into its own section to make it
easier on your eyes:

```
[dependencies.iced_widgets]
git = "https://github.com/manokara/iced_widgets"
default-features = false
features = ["hexview"]
```

## Widget List

Each widget below is listed as `Type name (feature name)`. Read the Optional Widgets section above
to know more about features.

- `Hexview` (`hexview`): A view into binary data. It has quite a few styling options, you can move a
  cursor around and also select things, but that doesn't do anything yet.

[iced]: https://github.com/hecrj/iced
