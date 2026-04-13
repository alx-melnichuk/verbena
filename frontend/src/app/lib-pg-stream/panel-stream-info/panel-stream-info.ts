import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, Output, ViewEncapsulation } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { TranslatePipe } from '@ngx-translate/core';
import { DateTimeFormatPipe } from '../../common/date-time-format-pipe';
import { Image } from "../../components/image/image";
import { StreamDto } from '../../lib-stream/stream-dto';

@Component({
    selector: 'app-panel-stream-info',
    exportAs: "appPanelStreamInfo",
    standalone: true,
    imports: [CommonModule, MatButtonModule, TranslatePipe, DateTimeFormatPipe, Image],
    templateUrl: './panel-stream-info.html',
    styleUrl: './panel-stream-info.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelStreamInfo {
    @Input()
    public canDuplicate = false;
    @Input()
    public canEdit = false;
    @Input()
    public canDelete = false;
    @Input()
    public isFuture = false;
    @Input()
    public locale: string | null = null;
    @Input()
    public streamDto: StreamDto | null | undefined;

    @Output()
    readonly requestNextPage: EventEmitter<void> = new EventEmitter();
    @Output()
    readonly actionDuplicate: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly actionEdit: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly actionView: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly actionDelete: EventEmitter<{ id: number, title: string }> = new EventEmitter();

    readonly formatDateTime: Intl.DateTimeFormatOptions = { dateStyle: "long", timeStyle: "short" };

    @HostBinding("class.app-pn-bg")
    public get isPnBg(): boolean { return true; }

    // ** Public API **

    public trackByIdFn(index: number, item: StreamDto): number {
        return item.id;
    }

    public doActionDuplicate(streamId?: number | null | undefined): void {
        if (!!streamId) {
            this.actionDuplicate.emit(streamId);
        }
    }
    public doActionEdit(streamId?: number | null | undefined): void {
        if (!!streamId) {
            this.actionEdit.emit(streamId);
        }
    }

    public doActionView(streamId?: number | null | undefined): void {
        if (!!streamId) {
            this.actionView.emit(streamId);
        }
    }

    public doActionDelete(streamDto?: StreamDto | null | undefined): void {
        if (!!streamDto) {
            this.actionDelete.emit({ id: streamDto.id, title: streamDto.title });
        }
    }

    // ** Private API **

}
