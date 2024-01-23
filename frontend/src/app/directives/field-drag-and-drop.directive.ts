import { Directive, EventEmitter, HostBinding, HostListener, Output } from '@angular/core';

@Directive({
  selector: '[appFieldDragAndDrop]',
  standalone: true
})
export class FieldDragAndDropDirective {

  @Output()
  readonly fileDropped: EventEmitter<FileList> = new EventEmitter();

  constructor() { 
    console.log(`FieldDragAndDropDirective()`);
  }

  @HostBinding('class.fileover')
  public fileOver = false;

  // Dragover listener
  @HostListener('dragover', ['$event'])
  public onDragOver(evt: DragEvent): void {
    evt.preventDefault();
    evt.stopPropagation();
    this.fileOver = true;
  }

  // Dragleave listener
  @HostListener('dragleave', ['$event'])
  public onDragLeave(evt: DragEvent): void {
    evt.preventDefault();
    evt.stopPropagation();
    this.fileOver = false;
  }

  // Drop listener
  @HostListener('drop', ['$event'])
  public ondrop(evt: DragEvent): void {
    evt.preventDefault();
    evt.stopPropagation();
    this.fileOver = false;
    const files = evt.dataTransfer?.files;
    if (!!files && files.length > 0) {
      this.fileDropped.emit(files);
    }
  }
}
