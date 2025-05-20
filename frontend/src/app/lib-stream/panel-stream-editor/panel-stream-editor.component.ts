import {
    ChangeDetectionStrategy, Component, ElementRef, EventEmitter, HostBinding, Input, OnChanges, Output,
    SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { ReactiveFormsModule, FormControl, FormGroup } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatChipsModule } from '@angular/material/chips';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatTooltipModule } from '@angular/material/tooltip';
import { TranslatePipe } from '@ngx-translate/core';

import { MAX_FILE_SIZE, IMAGE_VALID_FILE_TYPES } from 'src/app/common/constants';
import { StringDateTime } from 'src/app/common/string-date-time';
import { FieldChipGridComponent } from 'src/app/components/field-chip-grid/field-chip-grid.component';
import { FieldDateComponent } from 'src/app/components/field-date/field-date.component';
import { FieldDescriptComponent } from 'src/app/components/field-descript/field-descript.component';
import { FieldImageAndUploadComponent } from 'src/app/components/field-image-and-upload/field-image-and-upload.component';
import { FieldTimeComponent } from 'src/app/components/field-time/field-time.component';
import { FieldTitleComponent } from 'src/app/components/field-title/field-title.component';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { ClipboardUtil } from 'src/app/utils/clipboard.util';
import { FileSizeUtil } from 'src/app/utils/file_size.util';
import { HtmlElemUtil } from 'src/app/utils/html-elem.util';
import { ValidFileTypesUtil } from 'src/app/utils/valid_file_types.util';
import { TimeUtil } from 'src/app/utils/time.util';

import { StreamService } from '../stream.service';
import { StreamDto, StreamDtoUtil, UpdateStreamFileDto } from '../stream-api.interface';
import { StreamConfigDto } from '../stream-config.interface';

export const PSE_LOGO_MX_HG = '---pse-logo-mx-hg';
export const PSE_LOGO_MX_WD = '---pse-logo-mx-wd';

@Component({
    selector: 'app-panel-stream-editor',
    exportAs: 'appPanelStreamEditor',
    standalone: true,
    imports: [
        CommonModule, ReactiveFormsModule, MatButtonModule, MatChipsModule, MatFormFieldModule, MatInputModule, MatSlideToggleModule,
        MatTooltipModule, TranslatePipe, FieldDescriptComponent, FieldChipGridComponent, FieldImageAndUploadComponent,
        FieldTimeComponent, FieldTitleComponent, FieldDateComponent,
    ],
    templateUrl: './panel-stream-editor.component.html',
    styleUrl: './panel-stream-editor.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamEditorComponent implements OnChanges {
    @Input()
    public isDisabledSubmit = false;
    @Input()
    public locale: string | null = null;
    @Input()
    public streamDto: StreamDto | null = null;
    @Input()
    public streamConfigDto: StreamConfigDto | null = null;
    @Input()
    public errMsgs: string[] = [];

    @Output()
    readonly changeData: EventEmitter<void> = new EventEmitter();
    @Output()
    readonly updateStream: EventEmitter<UpdateStreamFileDto> = new EventEmitter();

    @HostBinding('class.global-scroll')
    public get isGlobalScroll(): boolean { return true; }

    public minDate: Date = new Date(Date.now());
    public maxDate: Date = new Date(this.minDate.getFullYear(), this.minDate.getMonth() + 7, 0);

    // FieldImageAndUpload parameters
    public accepts = IMAGE_VALID_FILE_TYPES;
    public maxSize = MAX_FILE_SIZE;
    public availableFileTypes: string = '';
    public availableMaxFileSize: string = '';

    // FieldImageAndUpload FormControl
    public logoFile: File | null | undefined;
    public initIsLogo: boolean = false; // original has an logo.

    readonly tagMaxLength: number = 255;
    readonly tagMinLength: number = 2;
    readonly tagMaxAmount: number = 4;
    readonly tagMinAmount: number = 1;
    readonly isTagRemovable = true;

    public isCreate = true;

    public controls = {
        title: new FormControl(null, []),
        descript: new FormControl(null, []),
        logo: new FormControl('', []),
        tags: new FormControl([], []),
        isStartTime: new FormControl(false, []),
        startDate: new FormControl({ value: new Date(Date.now()), disabled: true }, []),
        startTime: new FormControl('', []),
        link: new FormControl('', []),
    };

    public linkForVisitors = '';

    public formGroup: FormGroup = new FormGroup(this.controls);

    private origStreamDto: StreamDto = StreamDtoUtil.create();
    private isChange: boolean = false;

    constructor(
        public hostRef: ElementRef<HTMLElement>,
        private alertService: AlertService,
        private streamService: StreamService,
    ) {
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['streamDto']) {
            this.prepareFormGroupByStreamDto(this.streamDto);
        }
        if (!!changes['streamConfigDto']) {
            this.prepareFormGroupByStreamConfigDto(this.streamConfigDto);
            this.settingProperties(this.hostRef, this.streamConfigDto);
        }
    }

    // ** Public API **

    // ** Logo file **
    public addLogoFile(file: File): void {
        this.logoFile = file;
    }
    public deleteLogoFile(): void {
        this.logoFile = (!!this.initIsLogo ? null : undefined);
        if (!!this.initIsLogo) {
            this.controls.logo.markAsDirty();
        } else {
            this.controls.logo.markAsPristine();
        }
    }

    public changeIsStartTime(): void {
        const isStartTime: boolean = !!this.controls.isStartTime.value;
        if (!isStartTime) {
            this.controls.startDate.disable({ emitEvent: false });
            this.controls.startTime.disable();
        } else {
            this.controls.startDate.enable({ emitEvent: false });
            this.controls.startTime.enable();
        }
    }

    public updateErrMsgs(errMsgs: string[] = []): void {
        this.errMsgs = errMsgs;
    }

    public doChangeData(): void {
        if (!this.isChange) {
            this.isChange = true;
            this.changeData.emit();
        }
    }

    public saveStream(formGroup: FormGroup): void {
        const cntlTitle = formGroup.get('title');
        const cntlDescript = formGroup.get('descript');
        const cntlTags = formGroup.get('tags');
        const cntlIsStartTime = formGroup.get('isStartTime');
        const cntlStartDate = formGroup.get('startDate');
        const cntlStartTime = formGroup.get('startTime');
        if (formGroup.pristine || formGroup.invalid || !cntlTitle || !cntlDescript || !cntlTags || !cntlIsStartTime
            || !cntlStartDate || !cntlStartTime) {
            return;
        }

        const title: string = cntlTitle.value || '';
        const descript: string = cntlDescript.value || '';
        let starttime: StringDateTime | undefined;
        if (!!cntlIsStartTime.value) {
            const startDateTime = this.getStartDateTime(this.controls.startDate.value, this.controls.startTime.value);
            starttime = !!startDateTime ? startDateTime.toISOString() : undefined;
        }
        const tags: string[] = cntlTags.value || [];

        const updateStreamFileDto: UpdateStreamFileDto = {
            id: (this.isCreate ? undefined : this.streamDto?.id),
            title: (this.isCreate ? title : (this.origStreamDto.title != title ? title : undefined)),
            descript: (this.isCreate ? descript : (this.origStreamDto.descript != descript ? descript : undefined)),
            starttime: (this.isCreate ? starttime : (this.origStreamDto.starttime != starttime ? starttime : undefined)),
            tags: (this.isCreate ? tags : (this.origStreamDto.tags.join(',') != tags.join(',') ? tags : undefined)),
            logoFile: this.logoFile,
        };
        this.updateStream.emit(updateStreamFileDto);
    }

    public doCopyToClipboard(value: string): void {
        if (!!value) {
            ClipboardUtil.copyMessage(value);
            this.alertService.showInfo('panel-stream-editor.stream_link_copied_to_clipboard');
        }
    }

    // ** Private API **

    private prepareFormGroupByStreamDto(streamDto: StreamDto | null): void {
        const isDuplicate = streamDto?.id == -1;
        if (!streamDto) {
            streamDto = StreamDtoUtil.create();
        }
        this.origStreamDto = { ...streamDto };
        Object.freeze(this.origStreamDto);

        this.isCreate = (streamDto.id < 0);
        const now = new Date(Date.now())
        const currentTime = new Date(now.getFullYear(), now.getMonth(), now.getDate(), now.getHours(), now.getMinutes() + 5, now.getSeconds());
        // Date.parse("2010-01-01T00:00:00.000Z");
        const startDate = (!!streamDto.starttime ? new Date(Date.parse(streamDto.starttime)) : currentTime);
        const startHours = ('00' + startDate.getHours()).slice(-2);
        const startMinutes = ('00' + startDate.getMinutes()).slice(-2);
        const startTimeStr = startHours + ':' + startMinutes;
        const link = !this.isCreate ? this.streamService.getLinkForVisitors(streamDto.id, true) : '';
        // Marks all descendants of `FormGroup` as `pristine` and `untouched` and sets the initial values.
        this.formGroup.reset({
            title: streamDto.title,
            descript: streamDto.descript,
            logo: streamDto.logo,
            tags: (streamDto.tags || []),
            starttime: streamDto.starttime,
            isStartTime: (streamDto.id > 0 && !!streamDto.starttime),
            startDate: startDate,
            startTime: startTimeStr,
            link: link,
        });
        this.linkForVisitors = link;
        this.changeIsStartTime();
        this.logoFile = undefined;
        this.initIsLogo = !!streamDto.logo;
        if (isDuplicate) {
            this.formGroup.markAsDirty();
        }
        this.isChange = false;
    }
    // '10:12'
    private getStartDateTime(startDate: Date | null, startTime: string | null): Date | null {
        let startDateTime: Date | null = null;
        if (startDate != null) {
            startDateTime = new Date(startDate.getFullYear(), startDate.getMonth(), startDate.getDate(), 0, 0, 0, 0);
        }
        if (startDateTime != null && startTime != null && startTime.length > 4) {
            let { hours, minutes } = TimeUtil.parseTimeHHMM(startTime);
            startDateTime.setHours(hours);
            startDateTime.setMinutes(minutes);
            startDateTime.setSeconds(0);
            startDateTime.setMilliseconds(0);
        }
        return startDateTime;
    }
    private prepareFormGroupByStreamConfigDto(streamConfigDto: StreamConfigDto | null): void {
        // Set FieldImageAndUpload parameters
        this.maxSize = streamConfigDto?.logoMaxSize || MAX_FILE_SIZE;
        this.accepts = (streamConfigDto?.logoValidTypes || []).join(',');
        this.availableFileTypes = ValidFileTypesUtil.text(this.accepts).join(', ').toUpperCase();
        this.availableMaxFileSize = FileSizeUtil.formatBytes(this.maxSize, 1);
    }
    private settingProperties(elem: ElementRef<HTMLElement> | null, streamConfigDto: StreamConfigDto | null): void {
        const avatarMaxWidth = streamConfigDto?.logoMaxWidth;
        const maxWidth = (avatarMaxWidth && avatarMaxWidth > 0 ? avatarMaxWidth : undefined)
        HtmlElemUtil.setProperty(elem, PSE_LOGO_MX_WD, maxWidth?.toString().concat('px'));

        const avatarMaxHeight = streamConfigDto?.logoMaxHeight;
        const maxHeight = (avatarMaxHeight && avatarMaxHeight > 0 ? avatarMaxHeight : undefined)
        HtmlElemUtil.setProperty(elem, PSE_LOGO_MX_HG, maxHeight?.toString().concat('px'));
    }
}
