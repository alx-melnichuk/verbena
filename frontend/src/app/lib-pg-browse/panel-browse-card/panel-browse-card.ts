import { CommonModule } from '@angular/common';
import {
    AfterContentInit, ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, inject, Input, OnChanges, Output, SimpleChanges,
    ViewEncapsulation
} from '@angular/core';
import { TranslatePipe } from '@ngx-translate/core';
import { StreamDto, StreamState } from '../../lib-stream/stream-dto';
import { StringUtil } from '../../utils/string.util';
import { PanelBrowseActions } from '../panel-browse-actions/panel-browse-actions';
import { PanelBrowseParams } from '../panel-browse-params/panel-browse-params';

@Component({
    selector: 'app-panel-browse-card',
    exportAs: "appPanelBrowseCard",
    standalone: true,
    imports: [CommonModule, TranslatePipe, PanelBrowseActions, PanelBrowseParams],
    templateUrl: './panel-browse-card.html',
    styleUrl: './panel-browse-card.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelBrowseCard implements AfterContentInit, OnChanges {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);

    @Input()
    public countOfViewer: number | null | undefined;
    @Input()
    public locale: string | null = null;
    @Input()
    public isStreamOwner: boolean = false;
    @Input()
    public ownerAvatar: string | null | undefined;
    @Input()
    public ownerEmail: string | null | undefined;
    @Input()
    public ownerNickname: string | null | undefined;
    @Input()
    public streamDto: StreamDto | null = null;
    @Input()
    public timerActive: boolean | null | undefined;
    @Input()
    public timerIsShow: boolean | null | undefined;
    @Input()
    public timerValue: number | null | undefined;

    @Output()
    readonly changeState: EventEmitter<StreamState> = new EventEmitter();

    // To disable the jumping effect of the "stream-video" panel at startup.
    public isStreamVideo = false;
    public ownerNameSymbols: string = "";

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["ownerNickname"]) {
            this.ownerNameSymbols = (StringUtil.capitalizeOnlyFirstLetter(this.ownerNickname) || "").slice(0, 2);
        }
    }

    // To disable the jumping effect of the "stream-video" panel at startup.
    ngAfterContentInit(): void {
        this.isStreamVideo = true;
        this.changeDetector.markForCheck();
    }

    // ** Public API **

    // Section: "panel stream admin"

    public doChangeState(newState: StreamState): void {
        this.changeState.emit(newState);
    }

    // ** Private API **
}
