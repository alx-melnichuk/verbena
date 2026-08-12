import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamList } from './panel-stream-list';

describe('PanelStreamList', () => {
  let component: PanelStreamList;
  let fixture: ComponentFixture<PanelStreamList>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelStreamList]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelStreamList);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
