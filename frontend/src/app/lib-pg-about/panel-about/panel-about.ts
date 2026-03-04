import { CommonModule } from "@angular/common";
import { Component, ViewEncapsulation, ChangeDetectionStrategy, OnChanges, Input, HostBinding, SimpleChanges } from "@angular/core";
import { TranslatePipe } from "@ngx-translate/core";

@Component({
    selector: "app-panel-about",
    exportAs: "appPanelAbout",
    standalone: true,
    imports: [CommonModule, TranslatePipe],
    templateUrl: "./panel-about.html",
    styleUrl: "./panel-about.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelAbout implements OnChanges {
    @Input()
    public backendItem01: string | null | undefined;
    @Input()
    public backendItem02: string[] | null | undefined;
    @Input()
    public backendItem03: string[] | null | undefined;

    @HostBinding("class.global-scroll")
    public get isGlobalScroll(): boolean { return true; }

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
