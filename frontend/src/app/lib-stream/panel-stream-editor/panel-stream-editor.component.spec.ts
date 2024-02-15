import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamEditorComponent } from './panel-stream-editor.component';

describe('PanelStreamEditorComponent', () => {
  let component: PanelStreamEditorComponent;
  let fixture: ComponentFixture<PanelStreamEditorComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelStreamEditorComponent]
    });
    fixture = TestBed.createComponent(PanelStreamEditorComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
