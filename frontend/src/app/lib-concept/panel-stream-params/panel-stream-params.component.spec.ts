import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamParamsComponent } from './panel-stream-params.component';

describe('PanelStreamParamsComponent', () => {
  let component: PanelStreamParamsComponent;
  let fixture: ComponentFixture<PanelStreamParamsComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelStreamParamsComponent]
    });
    fixture = TestBed.createComponent(PanelStreamParamsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
