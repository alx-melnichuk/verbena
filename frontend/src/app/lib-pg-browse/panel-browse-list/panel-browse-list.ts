import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, Component, ViewEncapsulation } from "@angular/core";

@Component({
    selector: "app-panel-browse-list",
    exportAs: "appPanelBrowseList",
    standalone: true,
    imports: [CommonModule],
    templateUrl: "./panel-browse-list.html",
    styleUrl: "./panel-browse-list.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelBrowseList {

}
