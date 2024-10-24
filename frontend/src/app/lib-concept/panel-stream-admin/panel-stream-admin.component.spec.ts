import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamAdminComponent } from './panel-stream-admin.component';

describe('PanelStreamAdminComponent', () => {
  let component: PanelStreamAdminComponent;
  let fixture: ComponentFixture<PanelStreamAdminComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelStreamAdminComponent]
    });
    fixture = TestBed.createComponent(PanelStreamAdminComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
