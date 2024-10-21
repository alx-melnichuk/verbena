import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamInfoOwnerComponent } from './panel-stream-info-owner.component';

describe('PanelStreamInfoOwnerComponent', () => {
  let component: PanelStreamInfoOwnerComponent;
  let fixture: ComponentFixture<PanelStreamInfoOwnerComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelStreamInfoOwnerComponent]
    });
    fixture = TestBed.createComponent(PanelStreamInfoOwnerComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
