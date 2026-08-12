import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, Component, HostBinding, Input, ViewEncapsulation } from "@angular/core";
import { TranslatePipe } from "@ngx-translate/core";

/* 
  As with a block element, you must specify either the width or the height (or both).
  
  (1) The width and height dimensions correspond to the actual dimensions of the image.
    <app-image [srcImg]="image"></app-image>

  (2) - The height of the element is specified. The aspect ratio of the image is preserved.
    <app-image style="height: 200px;" [srcImg]="image"></app-image>

  (3) - The width of the element is specified. The aspect ratio of the image is preserved.
    <app-image style="width: 240px;" [srcImg]="image"></app-image>

  (4) - The width and height of the element are specified. The aspect ratio of the image is preserved.
    <app-image style="height: 200px; width: 240px;" [srcImg]="image"></app-image>

  
  You can specify the "i-shd" attribute to display a shadow on the image border.
    <app-image style="height: 200px; width: 240px;" i-shd [srcImg]="image"></app-image>

  You can specify the "border-radius" property to display rounding around the edges of the image.
    <app-image style="height: 200px; width: 240px; border-radius: 16px;" [srcImg]="image"></app-image>
*/
@Component({
    selector: "app-image",
    exportAs: "appImage",
    standalone: true,
    imports: [CommonModule, TranslatePipe],
    templateUrl: "./image.html",
    styleUrl: "./image.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Image {
    @Input()
    public srcImg: string | null | undefined;

    @HostBinding("attr.i-mn-sz")
    public get attrIsMinSize(): string | null {
        return !this.srcImg ? "" : null;
    }

}
