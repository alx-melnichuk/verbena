import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, EventEmitter, Input, OnChanges, Output, SimpleChanges, ViewEncapsulation
} from "@angular/core";
import { ReactiveFormsModule, FormControl, FormGroup } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule } from "@angular/material/input";
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { TranslatePipe } from "@ngx-translate/core";
import { CN_TAG } from "../../common/fields-consts";
import { Spinner } from "../../components/spinner/spinner";
import { ItemViewPage, ViewListByPages } from "../../components/view-list-by-pages/view-list-by-pages";
import { StreamTagDto } from "../../lib-stream/stream-dto";
import { PanelStreamCard } from "../panel-stream-card/panel-stream-card";

const CN_DEFAULT_LIMIT = 10;

@Component({
    selector: "app-panel-browse-list",
    exportAs: "appPanelBrowseList",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatFormFieldModule, MatInputModule, MatButtonModule, MatSlideToggleModule,
        Spinner, PanelStreamCard, ViewListByPages, TranslatePipe],
    templateUrl: "./panel-browse-list.html",
    styleUrl: "./panel-browse-list.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelBrowseList implements OnChanges {
    @Input()
    public isLive: boolean | null | undefined;
    @Input()
    public locale: string | null | undefined;
    @Input()
    public popTagIsLoading: boolean | null | undefined;
    @Input()
    public popTagList: StreamTagDto[] = [];
    @Input()
    public strmCrdIsLoading: boolean | null | undefined;
    @Input()
    public strmCrdIsReset: boolean | null | undefined;
    @Input()
    public strmCrdItemPage: ItemViewPage | null | undefined;
    @Input()
    public strmCrdMaxSizeRows: number = CN_DEFAULT_LIMIT * 3;
    @Input()
    public tag: string | null | undefined;

    @Output()
    readonly loadStrmCrdPage: EventEmitter<{ isLive: boolean, tag: string, page: number, limit: number }> = new EventEmitter();
    @Output()
    readonly actionView: EventEmitter<number> = new EventEmitter();

    readonly cn_tag = CN_TAG;

    public controls = {
        isLive: new FormControl(false, []),
        tag: new FormControl("", []),
    };
    public formGroup: FormGroup = new FormGroup(this.controls);

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["tag"]) {
            this.controls.tag.setValue(this.tag || "");
        }
        if (!!changes["isLive"]) {
            this.controls.isLive.setValue(!!this.isLive);
        }
    }

    public doSetTag(tag: string): void {
        this.controls.tag.setValue(tag);
    }

    public doActionView(streamId: number): void {
        this.actionView.emit(streamId);
    }

    public doLoadStrmCrdPage(isLive: boolean | null, tag: string | null, page: number, limit: number): void {
        this.loadStrmCrdPage.emit({ isLive: !!isLive, tag: (tag || ""), page, limit });
    }
}
