# Croppers
Gpu accelerated image viewer with streamlined cropping workflow and some fun image filters.


### Install
have [just](https://github.com/casey/just) and [rust](https://rust-lang.org/learn/get-started/) installed and run
```bash
just install
```
or look at the justfile and place the files there yourself.
Only tested and developed on arch/wayland/niri.

## Features

### different cropping modes
original and cropped version live side by side. That makes adjustments to the crop easy. Just toggling crop on again ('c') and the original image is shown.

#### Freeform Crop
<img src="assets/readme/crop_freeform.png" alt="Screenshot" width="600">

#### Crop to desired output dimensions
a crop with integrated scaling

<img src="/assets/readme/crop_to_fixed_dimensions.png" alt="Screenshot" width="600">

#### Various fixed aspect ratios and scaling


### Effect Stack
A gpu accelerated effect stack with some fun effects.

vignette

<img src="/assets/readme/effect_vignette.png" alt="Screenshot" width="600">

stacking effects with drag and drop reordering

<img src="/assets/readme/stacked_effects.png" alt="Screenshot" width="600">

<img src="/assets/readme/effect_ascii.png" alt="Screenshot" width="600">


### Settings for working fast

<table>
  <tr>
    <td width="70%" valign="top">
      <p>
          Set what the options for new images should be depending on the task currently desired.
      </p>
      <ul>
        <li>Fast setup</li>
        <li>Zero dependencies</li>
      </ul>
    </td>
    <td width="30%" align="center" valign="top">
      <img  src="/assets/readme/settings.png" alt="Screenshot" width="286">
    </td>
  </tr>
</table>
