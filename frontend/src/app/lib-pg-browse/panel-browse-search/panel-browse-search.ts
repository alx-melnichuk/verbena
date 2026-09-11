import { CommonModule } from '@angular/common';
import {
    ChangeDetectionStrategy, Component, EventEmitter, Input, OnChanges, Output, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { TranslatePipe } from '@ngx-translate/core';
import { CN_TAG } from '../../common/fields-consts';
import { StreamTagDto } from '../../lib-stream/stream-dto';

@Component({
    selector: 'app-panel-browse-search',
    exportAs: "appPanelBrowseList",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatFormFieldModule, MatInputModule, MatButtonModule, MatSlideToggleModule, TranslatePipe,],
    templateUrl: './panel-browse-search.html',
    styleUrl: './panel-browse-search.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelBrowseSearch implements OnChanges {
    @Input()
    public isLive: boolean | null | undefined;
    @Input()
    public popTagIsLoading: boolean | null | undefined;
    @Input()
    public popTagList: StreamTagDto[] = [];

    @Input()
    public tag: string | null | undefined;

    @Output()
    readonly loadStrmCrdPage: EventEmitter<{ isLive: boolean, tag: string, page: number, isReset: boolean }> = new EventEmitter();

    readonly cn_tag = CN_TAG;

    public cntls = {
        isLive: new FormControl(false, []),
        tag: new FormControl("", []),
    };
    public formGroup: FormGroup = new FormGroup(this.cntls);

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["tag"]) {
            this.cntls.tag.setValue(this.tag || "");
        }
        if (!!changes["isLive"]) {
            this.cntls.isLive.setValue(!!this.isLive);
        }
    }

    // ** Public API **

    public doSetTag(tag: string): void {
        this.cntls.tag.setValue(tag);
    }

    public doLoadStrmCrdPage(isLive: boolean | null, tag: string | null, page: number, isReset: boolean): void {
        this.loadStrmCrdPage.emit({ isLive: !!isLive, tag: (tag || ""), page, isReset });
    }
}
