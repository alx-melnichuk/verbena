import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, Input, OnChanges, SimpleChanges, ViewEncapsulation } from '@angular/core';
import { TranslatePipe } from '@ngx-translate/core';

@Component({
    selector: 'app-panel-about-info',
    exportAs: "appPanelAboutInfo",
    standalone: true,
    imports: [CommonModule, TranslatePipe],
    templateUrl: './panel-about-info.html',
    styleUrl: './panel-about-info.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelAboutInfo implements OnChanges {
    @Input()
    public backendItem01: string | null | undefined;
    @Input()
    public backendItem02: string[] | null | undefined;
    @Input()
    public backendItem03: string[] | null | undefined;

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["backendItem01"]) {
            this.backendItem01 = this.backendItem01 || "";
        }
        if (!!changes["backendItem02"]) {
            this.backendItem02 = this.backendItem02 || [];
        }
        if (!!changes["backendItem03"]) {
            this.backendItem03 = this.backendItem03 || [];
        }
    }

    // ** Public API **

    public getKey(item: string): string {
        const itemVal = (item || "");
        const n = itemVal.indexOf("=");
        return n > -1 ? itemVal.slice(0, n).trim() : "";
    }

    // ** Private API **

}
