import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, Input, OnChanges, Output, 
  SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { ENTER } from '@angular/cdk/keycodes';
import { ReactiveFormsModule, FormControl, Validators, FormGroup, ValidationErrors, FormArray } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatChipsModule } from '@angular/material/chips';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatTooltipModule } from '@angular/material/tooltip';
import { TranslateModule } from '@ngx-translate/core';

import { MAX_FILE_SIZE, IMAGE_VALID_FILE_TYPES } from 'src/app/common/constants';
import { FieldChipGridComponent } from 'src/app/components/field-chip-grid/field-chip-grid.component';
import { FieldDescriptComponent } from 'src/app/components/field-descript/field-descript.component';
import { FieldFileUploadComponent } from 'src/app/components/field-file-upload/field-file-upload.component';
import { FieldTimeComponent } from 'src/app/components/field-time/field-time.component';
import { StringDateTime } from 'src/app/common/string-date-time';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { CopyToClipboardUtil } from 'src/app/utils/copy-to-clipboard.util';
import { TimeUtil } from 'src/app/utils/time.util';

import { StreamService } from '../stream.service';
import { StreamDto, StreamDtoUtil, UpdateStreamFileDto } from '../stream-api.interface';

@Component({
  selector: 'app-panel-stream-editor',
  standalone: true,
  imports: [
    CommonModule, MatButtonModule, MatChipsModule, MatFormFieldModule, MatInputModule,  MatSlideToggleModule,
    MatDatepickerModule, MatTooltipModule, TranslateModule, ReactiveFormsModule, FieldDescriptComponent, FieldChipGridComponent,
    FieldFileUploadComponent, FieldTimeComponent
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
  
  @Output()
  readonly updateStream: EventEmitter<UpdateStreamFileDto> = new EventEmitter();
  @Output()
  readonly cancelStream: EventEmitter<void> = new EventEmitter();
  
  public minLenTitle = 2;
  public maxLenTitle = 255;
  public minLenDescription = 2;
  public maxLenDescription = 2048;
  public countRowsDescription = 4;

  public minDate: Date = new Date(Date.now());
  public maxDate: Date = new Date(this.minDate.getFullYear(), this.minDate.getMonth() + 7, 0);

  public origLogo: string | null = '';
  public addedLogoView: string = '';
  public maxFileSize = MAX_FILE_SIZE;
  public validFileTypes = IMAGE_VALID_FILE_TYPES;
  public addedLogoFile: File | null | undefined;

  readonly separatorCodes: number[] = [ENTER];
  readonly tagMaxLength: number = 255;
  readonly tagMinLength: number = 2;
  readonly tagMaxQuantity: number = 4;
  readonly tagMinQuantity: number = 1;
  readonly isTagRemovable = true;

  public isCreate = true;
  
  public controls = {
    title: new FormControl(null,
      [Validators.required, Validators.minLength(this.minLenTitle), Validators.maxLength(this.maxLenTitle)]),
    descript: new FormControl(null, []),
    logo: new FormControl('', []),
    tags: new FormControl([], []),
    isStartTime: new FormControl(false, []),
    startDate: new FormControl({ value: new Date(Date.now()), disabled: true }, []),
    startTime: new FormControl('', []),
  };

  public linkForVisitors = '';

  public formGroup: FormGroup = new FormGroup(this.controls);

  @HostBinding('class.global-scroll')
  public get isGlobalScroll(): boolean { return true; }
  

  private origStreamDto: StreamDto = StreamDtoUtil.create();
  
  constructor(
    private changeDetectorRef: ChangeDetectorRef,
    private alertService: AlertService,
    private streamService: StreamService,
  ) {
    console.log(`PanelStreamEditorComponent()`); // #-
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['streamDto'] && !!this.streamDto) {
      this.prepareData(this.streamDto);
    }
  }
  
  // ** Public API **

  public getErrorMsg(errors: ValidationErrors | null, name: string): string {
    let result: string = '';
    const errorsProps: string[] = errors != null ? Object.keys(errors) : [];
    for (let index = 0; index < errorsProps.length && !result; index++) {
      const error: string = errorsProps[index];
      result = !result && 'required' === error ? `Validation.${name}:required` : result;
      result = !result && 'minlength' === error ? `Validation.${name}:min_length` : result;
      result = !result && 'maxlength' === error ? `Validation.${name}:max_length` : result;
    }
    return result;
  }

  public addFile(file: File): void {
    this.addedLogoFile = file;
    this.controls.logo.setValue(file.name);
    this.controls.logo.markAsDirty();
  }

  public readFile(buffFile: string[]): void {
    if (buffFile.length > 0) {
      this.addedLogoView = buffFile[1];
      this.changeDetectorRef.markForCheck();
    }
  }

  public deleteFileLogo(): void {
    this.addedLogoFile = (!!this.origLogo ? null : undefined);
    this.addedLogoView = '';
    this.controls.logo.setValue(null);
    if (!!this.origLogo) {
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

  public saveStreamCard(): void {
    let starttime: StringDateTime | undefined;
    const isStartTime: boolean = !!this.controls.isStartTime.value;
    if (isStartTime) {
      const startDateTime = this.getStartDateTime(this.controls.startDate.value, this.controls.startTime.value);
      starttime = !!startDateTime ? startDateTime.toISOString() : undefined;
    }
    const title: string | undefined = this.controls.title.value || undefined;
    const descript: string | undefined = this.controls.descript.value || undefined;
    const tags = (this.controls.tags.value || []);

    const updateStreamFileDto: UpdateStreamFileDto = {};

    if (this.isCreate) { // Mode: "create"
      updateStreamFileDto.createStreamDto = {
        title: (title || ''),
        descript,
        starttime,
        tags,
      };
    } else { // Mode: "update"
      updateStreamFileDto.id = this.streamDto.id;
      updateStreamFileDto.modifyStreamDto = {
        title: (this.controls.title.dirty ? title : undefined),
        descript: (this.controls.descript.dirty ? descript : undefined),
        starttime: (this.controls.startDate.dirty || this.controls.startTime.dirty ? starttime : undefined),
        tags: (this.controls.tags.dirty ? tags : undefined),
      }
    }
    updateStreamFileDto.logoFile = this.addedLogoFile;
    this.updateStream.emit(updateStreamFileDto);
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
    const now = new Date(Date.now())
    const currentTime = new Date(now.getFullYear(), now.getMonth(), now.getDate(), now.getHours(), now.getMinutes() + 5, now.getSeconds());
    // Date.parse("2019-01-01T00:00:00.000Z");
    const startDate = (!!streamDto.starttime ? new Date(Date.parse(streamDto.starttime)) : currentTime);
    const startHours = ('00' + startDate.getHours()).slice(-2);
    const startMinutes = ('00' + startDate.getMinutes()).slice(-2);
    const startSeconds = ('00' + startDate.getSeconds()).slice(-2);
    const startTimeStr = startHours + ':' + startMinutes + ':' + startSeconds;
    this.formGroup.patchValue({
      title: streamDto.title,
      descript: streamDto.descript,
      logo: streamDto.logo,
      tags: streamDto.tags,
      starttime: streamDto.starttime,
      isStartTime: (streamDto.id > 0 && !!streamDto.starttime),
      startDate: startDate,
      startTime: startTimeStr,
    });
    this.linkForVisitors = this.streamService.getLinkForVisitors(streamDto.id, true);
    this.changeIsStartTime();
    this.addedLogoView = streamDto.logo || '';
    this.origLogo = streamDto.logo;
    this.addedLogoFile = undefined;
    this.isCreate = (streamDto.id < 0);
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
