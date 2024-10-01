import {
  ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, OnChanges, Output, 
  SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { ENTER } from '@angular/cdk/keycodes';
import { ReactiveFormsModule, FormControl, FormGroup  } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatChipsModule } from '@angular/material/chips';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatTooltipModule } from '@angular/material/tooltip';
import { TranslateModule } from '@ngx-translate/core';

import { MAX_FILE_SIZE, IMAGE_VALID_FILE_TYPES } from 'src/app/common/constants';
import { FieldChipGridComponent } from 'src/app/components/field-chip-grid/field-chip-grid.component';
import { FieldDateComponent } from 'src/app/components/field-date/field-date.component';
import { FieldDescriptComponent } from 'src/app/components/field-descript/field-descript.component';
import { FieldFileUploadComponent } from 'src/app/components/field-file-upload/field-file-upload.component';
import { FieldImageAndUploadComponent } from 'src/app/components/field-image-and-upload/field-image-and-upload.component';
import { FieldTimeComponent } from 'src/app/components/field-time/field-time.component';
import { FieldTitleComponent } from 'src/app/components/field-title/field-title.component';
import { StringDateTime } from 'src/app/common/string-date-time';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { CopyToClipboardUtil } from 'src/app/utils/copy-to-clipboard.util';
import { TimeUtil } from 'src/app/utils/time.util';

import { StreamService } from '../stream.service';
import { StreamDto, StreamDtoUtil, UpdateStreamFileDto } from '../stream-api.interface';

@Component({
  selector: 'app-panel-stream-editor',
  exportAs: 'appPanelStreamEditor',
  standalone: true,
  imports: [
    CommonModule, MatButtonModule, MatChipsModule, MatFormFieldModule, MatInputModule,  MatSlideToggleModule,
    MatTooltipModule, TranslateModule, ReactiveFormsModule, FieldDescriptComponent, FieldChipGridComponent,
    FieldImageAndUploadComponent, FieldFileUploadComponent, FieldTimeComponent, FieldTitleComponent, FieldDateComponent,
  ],
  templateUrl: './panel-stream-editor.component.html',
  styleUrls: ['./panel-stream-editor.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamEditorComponent implements OnChanges {
  @Input()
  public isDisabledSubmit = false;
  @Input()
  public streamDto: StreamDto = StreamDtoUtil.create();
  @Input()
  public errMsgs: string[] = [];
  
  @Output()
  readonly updateStream: EventEmitter<UpdateStreamFileDto> = new EventEmitter();
  @Output()
  readonly cancelStream: EventEmitter<void> = new EventEmitter();
  
  @HostBinding('class.global-scroll')
  public get isGlobalScroll(): boolean { return true; }

  public minDate: Date = new Date(Date.now());
  public maxDate: Date = new Date(this.minDate.getFullYear(), this.minDate.getMonth() + 7, 0);

  // FieldImageAndUpload parameters
  public accepts = IMAGE_VALID_FILE_TYPES;
  public maxSize = MAX_FILE_SIZE;
  // FieldImageAndUpload FormControl
  public logoFile: File | null | undefined;
  public initIsLogo: boolean = false; // original has an logo.
  
  readonly separatorCodes: number[] = [ENTER];
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
  
  constructor(
    private alertService: AlertService,
    private streamService: StreamService,
  ) {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['streamDto']) {
      this.prepareData(this.streamDto);
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

  public cancelStreamCard(): void {
    this.cancelStream.emit();
  }

  public updateErrMsgs(errMsgs: string[] = []): void {
    this.errMsgs = errMsgs;
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
      id: (this.isCreate ? undefined : this.streamDto.id),
      title: (this.isCreate ? title : (this.origStreamDto.title != title ? title : undefined)),
      descript: (this.isCreate ? descript : (this.origStreamDto.descript != descript ? descript : undefined)),
      starttime: (this.isCreate ? starttime : (this.origStreamDto.starttime != starttime ? starttime : undefined)),
      tags: (this.isCreate ? tags : (this.origStreamDto.tags.join(',') != tags.join(',') ? tags : undefined)),
      logoFile: this.logoFile,
    };

    const is_all_empty = Object.values(updateStreamFileDto).findIndex((value) => value !== undefined) == -1;
    if (!is_all_empty) {
      this.updateStream.emit(updateStreamFileDto);
    }
  }

  public doCopyToClipboard(value: string): void {
    if (!!value) {
      CopyToClipboardUtil.copyMessage(value);
      this.alertService.showInfo('stream_edit.stream_link_copied_to_clipboard');
    }
  }

  // ** Private API **

  private prepareData(streamDto: StreamDto): void {
    if (!streamDto) {
      return;
    }
    this.origStreamDto = { ...streamDto };
    Object.freeze(this.origStreamDto);
    this.isCreate = (streamDto.id < 0);
    const now = new Date(Date.now())
    const currentTime = new Date(now.getFullYear(), now.getMonth(), now.getDate(), now.getHours(), now.getMinutes() + 5, now.getSeconds());
    // Date.parse("2019-01-01T00:00:00.000Z");
    const startDate = (!!streamDto.starttime ? new Date(Date.parse(streamDto.starttime)) : currentTime);
    const startHours = ('00' + startDate.getHours()).slice(-2);
    const startMinutes = ('00' + startDate.getMinutes()).slice(-2);
    const startSeconds = ('00' + startDate.getSeconds()).slice(-2);
    const startTimeStr = startHours + ':' + startMinutes + ':' + startSeconds;
    const link = !this.isCreate ? this.streamService.getLinkForVisitors(streamDto.id, true) : '';
    this.formGroup.patchValue({
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
}
