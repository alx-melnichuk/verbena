import {
    ChangeDetectionStrategy, Component, EventEmitter, inject, Input, Output, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';

import { LocaleService } from 'src/app/common/locale.service';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';

import { PanelStreamEditorComponent } from '../panel-stream-editor/panel-stream-editor.component';
import { StreamDto, UpdateStreamFileDto } from '../stream-api.interface';
import { StreamConfigDto } from '../stream-config.interface';

@Component({
    selector: 'app-stream-edit',
    standalone: true,
    imports: [CommonModule, SpinnerComponent, PanelStreamEditorComponent],
    templateUrl: './stream-edit.component.html',
    styleUrl: './stream-edit.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class StreamEditComponent {
    @Input()
    public errMsgs: string[] = [];
    @Input()
    public isLoadStream = false;
    @Input()
    public streamDto: StreamDto | null = null;
    @Input()
    public streamConfigDto: StreamConfigDto | null = null;

    @Output()
    readonly changeData: EventEmitter<void> = new EventEmitter();
    @Output()
    readonly updateStream: EventEmitter<UpdateStreamFileDto> = new EventEmitter();

    public localeService: LocaleService = inject(LocaleService);

    // ** Public API **

    public doChangeData(): void {
        this.changeData.emit();
    }

    public doUpdateStream(updateStreamFileDto: UpdateStreamFileDto | null): void {
        if (!!updateStreamFileDto) {
            this.updateStream.emit(updateStreamFileDto);
        }
    }

    // ** Private API **

}
