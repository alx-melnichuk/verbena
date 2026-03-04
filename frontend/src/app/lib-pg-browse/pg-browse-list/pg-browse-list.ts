import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, Component, OnInit, ViewEncapsulation } from "@angular/core";
import { PanelBrowseList } from "../panel-browse-list/panel-browse-list";

@Component({
    selector: "app-pg-browse-list",
    exportAs: "appPgBrowseList",
    standalone: true,
    imports: [CommonModule, PanelBrowseList],
    templateUrl: "./pg-browse-list.html",
    styleUrl: "./pg-browse-list.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgBrowseList implements OnInit {

    ngOnInit(): void {
        console.log(`PgBrowseList.OnInit()`);
    }

    // ** Public API **

}
